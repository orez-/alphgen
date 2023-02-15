use alphgen;

fn main() {
    let width = 8;
    let height = 8;
    let glyphs: [(char, &[u8]); 26] = [
        ('a', &0x0000708888986800u64.to_be_bytes()),
        ('b', &0x8080f0888888f000u64.to_be_bytes()),
        ('c', &0x0000708880887000u64.to_be_bytes()),
        ('d', &0x0808788888887800u64.to_be_bytes()),
        ('e', &0x00007088f8807000u64.to_be_bytes()),
        ('f', &0x1028202070202000u64.to_be_bytes()),
        ('g', &0x0000708888780870u64.to_be_bytes()),
        ('h', &0x808080f088888800u64.to_be_bytes()),
        ('i', &0x0020002020202000u64.to_be_bytes()),
        ('j', &0x002000202020a040u64.to_be_bytes()),
        ('k', &0x4040485060504800u64.to_be_bytes()),
        ('l', &0x4040404040402000u64.to_be_bytes()),
        ('m', &0x000000d0a8a8a800u64.to_be_bytes()),
        ('n', &0x000000e090909000u64.to_be_bytes()),
        ('o', &0x0000708888887000u64.to_be_bytes()),
        ('p', &0x0000e09090e08080u64.to_be_bytes()),
        ('q', &0x0000709090701010u64.to_be_bytes()),
        ('r', &0x0000b0c880808000u64.to_be_bytes()),
        ('s', &0x00007080e010e000u64.to_be_bytes()),
        ('t', &0x0020207020201000u64.to_be_bytes()),
        ('u', &0x0000888888887800u64.to_be_bytes()),
        ('v', &0x0000888850502000u64.to_be_bytes()),
        ('w', &0x0000a4a474682800u64.to_be_bytes()),
        ('x', &0x0000885020508800u64.to_be_bytes()),
        ('y', &0x00009090907010e0u64.to_be_bytes()),
        ('z', &0x0000f8102040f800u64.to_be_bytes()),
    ];
    let ligatures = Vec::new();

    let font = alphgen::bitmap_font(width, height, glyphs, ligatures)
        .unwrap();
    font.save("my_neat_font.ttf")
        .unwrap();
}
