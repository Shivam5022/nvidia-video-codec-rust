#!/bin/bash
bindgen \
    --allowlist-type cudaVideo.* \
    --allowlist-type cuvid.* \
    --allowlist-type CUVID.* \
    --allowlist-function cuvid.* \
    --allowlist-var \[IPBS\]_VOP \
    --allowlist-var cuvid.* \
    --must-use-type CUresult \
    --must-use-type cuvidDecodeStatus \
    --blocklist-file .*/cuda\\.h \
    --blocklist-file .*/std.*\\.h \
    --default-enum-style=rust \
    --no-doc-comments \
    --with-derive-default \
    --with-derive-eq \
    --with-derive-hash \
    --with-derive-ord \
    --use-core \
    --merge-extern-blocks \
    --sort-semantically \
    --output cuviddec.rs ../headers/cuviddec.h -- -I/usr/local/cuda-12.8/include

bindgen \
    --allowlist-type CU.* \
    --allowlist-type cudaVideo.* \
    --allowlist-type cudaAudio.* \
    --allowlist-type HEVC.* \
    --allowlist-function cuvid.* \
    --allowlist-var MAX_CLOCK_TS \
    --blocklist-file .*/cuda\\.h \
    --blocklist-file .*/std.*\\.h \
    --blocklist-file .*/cuviddec\\.h \
    --must-use-type CUresult \
    \
    --default-enum-style=rust \
    --no-doc-comments \
    --with-derive-default \
    --with-derive-eq \
    --with-derive-hash \
    --with-derive-ord \
    --use-core \
    --merge-extern-blocks \
    --sort-semantically \
    --output nvcuvid.rs ../headers/nvcuvid.h -- -I/usr/local/cuda-12.8/include

bindgen \
    --allowlist-type NVENC.* \
    --allowlist-type NV_ENC.* \
    --allowlist-type NV_ENCODE.* \
    --allowlist-type GUID \
    --allowlist-type PENV.* \
    --allowlist-function NvEncodeAPI.* \
    --allowlist-function NvEnc.* \
    --allowlist-var NVENC.* \
    --allowlist-var NV_ENC.* \
    --allowlist-var NV_MAX.* \
    --blocklist-item NV_ENC_\\w+_GUID \
    --blocklist-file .*/cuda\\.h \
    --blocklist-file .*/std.*\\.h \
    --blocklist-file .*/cuviddec\\.h \
    --must-use-type NVENCSTATUS \
    \
    --default-enum-style=rust \
    --no-doc-comments \
    --with-derive-default \
    --with-derive-eq \
    --with-derive-hash \
    --with-derive-ord \
    --use-core \
    --merge-extern-blocks \
    --sort-semantically \
    --output nvEncodeAPI.rs ../headers/nvEncodeAPI.h -- -I/usr/local/cuda-12.8/include

# Additional preludes to make sure the bindings compile.
echo -e "use cudarc::driver::sys::*;\n$(cat cuviddec.rs)" > cuviddec.rs
echo -e "use super::cuviddec::*;\nuse cudarc::driver::sys::*;\ntype wchar_t = i32;\n$(cat nvcuvid.rs)" > nvcuvid.rs
echo -e "pub use super::super::version::*;\npub use super::super::guid::*;\n$(cat nvEncodeAPI.rs)" > nvEncodeAPI.rs
