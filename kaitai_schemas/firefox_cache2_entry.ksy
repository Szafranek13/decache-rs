meta:
  id: firefox_cache2_entry
  endian: be
seq:
  - id: data
    size: metadata_offset

instances:
  metadata_offset:
    pos: _io.size - 4
    type: u4

  metadata:
    pos: metadata_offset
    size: _io.size - metadata_offset - 4
