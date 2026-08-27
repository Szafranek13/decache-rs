meta:
  id: simple_cache
  endian: le

seq:
  - id: header
    type: cache_header

  - id: key
    size: header.key_length

instances:
  final_eof:
    pos: _io.size - 24
    type: cache_eof

#Apparently sometimes sha256 is present and sometimes not, i have no idea what determines that, and how to detect it for now :(
  has_key_sha256:
    value: (final_eof.flags & 2) != 0

  stream0_end_with_sha256:
    value: _io.size - 24 - 32
    if: has_key_sha256

  stream0_end_without_sha256:
    value: _io.size - 24
    if: not has_key_sha256

  key_sha256:
    pos: _io.size - 24 - 32
    size: 32
    if: has_key_sha256

  stream0_start_with_sha256:
    value: stream0_end_with_sha256 - final_eof.stream_size
    if: has_key_sha256

  stream0_start_without_sha256:
    value: stream0_end_without_sha256 - final_eof.stream_size
    if: not has_key_sha256

  stream0_with_sha256:
    pos: stream0_start_with_sha256
    size: final_eof.stream_size
    if: has_key_sha256

  stream0_without_sha256:
    pos: stream0_start_without_sha256
    size: final_eof.stream_size
    if: not has_key_sha256

  stream1_eof_with_sha256:
    pos: stream0_start_with_sha256 - 24
    type: cache_eof
    if: has_key_sha256

  stream1_eof_without_sha256:
    pos: stream0_start_without_sha256 - 24
    type: cache_eof
    if: not has_key_sha256

  stream1_start:
    value: 24 + header.key_length

  stream1_end_with_sha256:
    value: stream0_start_with_sha256 - 24
    if: has_key_sha256

  stream1_end_without_sha256:
    value: stream0_start_without_sha256 - 24
    if: not has_key_sha256

  stream1_with_sha256:
    pos: stream1_start
    size: stream1_end_with_sha256 - stream1_start
    if: has_key_sha256

  stream1_without_sha256:
    pos: stream1_start
    size: stream1_end_without_sha256 - stream1_start
    if: not has_key_sha256

types:
  cache_header:
    seq:
      - id: magic
        type: u8

      - id: version
        type: u4

      - id: key_length
        type: u4

      - id: key_hash
        type: u4

      - id: reserved
        type: u4

  cache_eof:
    seq:
      - id: magic
        type: u8

      - id: flags
        type: u4

      - id: crc32
        type: u4

      - id: stream_size
        type: u4

      - id: reserved
        type: u4
