# AlphGen

> Just "Alph", no "uh"

A (WIP) library for generating pixel fonts (.TTFs) from bitmaps.
An excuse to finally learn about font file formats.

My current goal for this library is to implement the following API:

```rust
let width = 8;
let height = 8;
let glyphs = hashmap! {
    'a': a_bitmap,
    'b': b_bitmap,
    'c': c_bitmap,
    // ...
};
let ligatures = hashmap! {
    "'d": apost_d_bitmap,
    // ...
}

let font = alphgen::bitmap_font(width, height, glyphs, ligatures);
font.save("my_neat_font.ttf");
```
