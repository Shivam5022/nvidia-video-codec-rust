# It will generate output_rust.bin
time cargo run --example decoder --release

# generating output_cpp.bin
time /home/satyam/dev/Video_Codec_SDK_13.0.19/Samples/build/AppDecode/AppDec/AppDec -i ~/dev/input.mp4 -o output_cpp.bin

# # Encoding the rust binary
/home/satyam/dev/Video_Codec_SDK_13.0.19/Samples/build/AppEncode/AppEncCuda/AppEncCuda -i output_rust.bin -o output_rust.h264 -s 1280x720 -if nv12

# Encoding the cpp binary
/home/satyam/dev/Video_Codec_SDK_13.0.19/Samples/build/AppEncode/AppEncCuda/AppEncCuda -i output_cpp.bin -o output_cpp.h264 -s 1280x720 -if nv12

ffmpeg -i output_rust.h264 -c:v copy output_rust.mp4
ffmpeg -i output_cpp.h264 -c:v copy output_cpp.mp4
