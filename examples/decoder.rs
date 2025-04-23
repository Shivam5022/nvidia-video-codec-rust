use cudarc::driver::CudaContext;
use ffmpeg::codec::Id;
use ffmpeg::color::Primaries;
use nvidia_video_codec_sdk::Decoder;
extern crate ffmpeg_next as ffmpeg;
use std::collections::VecDeque;
use std::ptr;

use ffmpeg::bsfilter::BSFContext;
use ffmpeg::{format, Packet};

fn demux(vid_path: &'static str) -> (VecDeque<Packet>, Id) {
    let ictx = format::input(&vid_path).unwrap();
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).unwrap();
    let stream_id = stream.id();
    println!("Format: {}", ictx.format().description());
    let is_mov = ictx.format().description() == "QuickTime / MOV";
    let is_flv = ictx.format().description() == "FLV (Flash Video)";
    let is_mkv = ictx.format().description() == "Matroska / WebM";
    let is_standard_format = is_mov && is_flv && is_mkv;

    // A video file might have many streams, select the "best" one.
    let codec_id = stream.parameters().id();
    let extra_data_size = unsafe { (*stream.parameters().as_ptr()).extradata_size } as usize;
    let extra_data: *mut u8 = unsafe { (*stream.parameters().as_ptr()).extradata };

    println!("Codec Id: {:?}", codec_id);
    let is_mp4h264 = codec_id == ffmpeg_next::codec::Id::H264 && !is_standard_format;
    let is_mp4hevc = codec_id == ffmpeg_next::codec::Id::HEVC && !is_standard_format;
    let is_mp4mpeg4 = codec_id == ffmpeg_next::codec::Id::MPEG4 && !is_standard_format;

    (
        if is_mp4h264 {
            let filter = BSFContext::new("h264_mp4toannexb", &stream.parameters()).unwrap();
            extract_packets_mp4h(ictx, stream_id, filter)
        } else if is_mp4hevc {
            let filter = BSFContext::new("hevc_mp4toannexb", &stream.parameters()).unwrap();
            extract_packets_mp4h(ictx, stream_id, filter)
        } else if is_mp4mpeg4 {
            extract_packets_mp4mpeg(ictx, stream_id, extra_data_size, extra_data)
        } else {
            println!("This else");
            extract_packets(ictx, stream_id)
        },
        codec_id,
    )
}

fn extract_packets_mp4mpeg(
    mut ictx: format::context::Input,
    stream_id: i32,
    extra_data_size: usize,
    extra_data: *mut u8,
) -> VecDeque<Packet> {
    // let mut filter = filter.unwrap();
    let mut packets: VecDeque<Packet> = VecDeque::new();
    for p in ictx.packets() {
        if p.0.id() == stream_id {
            let packet = if packets.len() == 0 {
                let packet_size = extra_data_size as usize + p.1.size() - 3 * size_of::<u8>();
                let mut buffer = vec![0u8; packet_size];
                let ptr: *mut u8 = buffer.as_mut_ptr();
                unsafe {
                    ptr::copy_nonoverlapping(extra_data, ptr, extra_data_size);
                    ptr::copy_nonoverlapping(
                        &p.1.data().unwrap()[3],
                        ptr.add(extra_data_size),
                        p.1.size() - 3,
                    );
                    Packet::copy(std::slice::from_raw_parts_mut(
                        ptr,
                        extra_data_size + p.1.size() - 3,
                    ))
                }
            } else {
                p.1
            };
            packets.push_back(packet);
        }
    }
    packets
}

fn extract_packets_mp4h(
    mut ictx: format::context::Input,
    stream_id: i32,
    mut filter: BSFContext,
) -> VecDeque<Packet> {
    let mut packets: VecDeque<Packet> = VecDeque::new();
    for p in ictx.packets() {
        if p.0.id() == stream_id {
            filter.send_packet(p.1).unwrap();
            let packet = filter.receive_packet().unwrap();
            packets.push_back(packet);
        }
    }
    packets
}

fn extract_packets(mut ictx: format::context::Input, stream_id: i32) -> VecDeque<Packet> {
    let mut packets: VecDeque<Packet> = VecDeque::new();
    for p in ictx.packets() {
        if p.0.id() == stream_id {
            packets.push_back(p.1);
        }
    }
    packets
}

fn main() {
    let vid_path = "/home/satyam/dev/nv_encdec/input.mp4";
    // Create a new CudaDevice to interact with cuda.
    // let cuda_device = CudaDevice::new(0).expect("Cuda should be installed correctly.");
    let ctx = CudaContext::new(0).unwrap();
    let stream = ctx.new_stream().unwrap();
    let (mut packets, codec_id) = demux(vid_path);
    let codec_id = ffmpeg_id_to_nv_id(codec_id);

    let mut decoder = Decoder::initialize_with_cuda(ctx.clone(), stream.clone(), codec_id)
        .expect("NVIDIA Video Codec SDK should be installed correctly.");

    let mut tot_frames = 0;
    loop {
        if packets.len() == 0 {
            break;
        }
        let mut packet = packets.pop_front().unwrap();
        let size = packet.size();
        let data = packet.data_mut().unwrap();
        let frame_returned = decoder.decode(data.as_mut_ptr(), size as u64);

        if frame_returned == 0 {
            if packets.len() == 0 {
                break;
            }
            continue;
        }
        for _ in 0..frame_returned {
            let p_frame = decoder.get_frame();
            println!("{:?}", p_frame);
            tot_frames += 1;
        }
    }

    println!("Total frames returned: {}", tot_frames);
}

fn ffmpeg_id_to_nv_id(codec_id: Id) -> nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec {
    match codec_id {
        Id::MPEG1VIDEO => {
            nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_MPEG1
        }
        Id::MPEG2VIDEO => {
            nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_MPEG2
        }
        Id::MPEG4 => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_MPEG4,
        Id::WMV3 | Id::VC1 => {
            nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_VC1
        }
        Id::H264 => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_H264,
        Id::HEVC => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_HEVC,
        Id::VP8 => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_VP8,
        Id::VP9 => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_VP9,
        Id::MJPEG => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_JPEG,
        Id::AV1 => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_AV1,
        _ => nvidia_video_codec_sdk::sys::cuviddec::cudaVideoCodec::cudaVideoCodec_NumCodecs,
    }
}
