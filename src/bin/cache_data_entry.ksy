meta:
  id: chromium_cache_data_entry
  endian: le
seq:
  - id: magick_number
    size: 4
  - id: version # perhaps?
    size: 4
  - id: key_len # not sure
    size: 4
  - id: key_hash # not sure
    size: 4
  - id: no_idea_what_this_is
    size: 4
  - id: no_idea_what_this_is2
    size: 3
  - id: no_idea_what_this_is3
    size: 6
  - id: no_idea_what_this_is4
    size: 1
  - id: no_idea_what_this_is5
    size: 2
  - id: urls
    size: 100
    type: str
    encoding: UTF-8
