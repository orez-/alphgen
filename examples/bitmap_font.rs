use alphgen;

fn main() {
    let width = 8;
    let height = 8;
    let ligatures = Vec::new();

    let glyphs = GLYPHS.into_iter().copied();
    let font = alphgen::bitmap_font(width, height, MISSING_GLYPH, glyphs, ligatures)
        .unwrap();
    font.save("my_neat_font.ttf")
        .unwrap();
}

const MISSING_GLYPH: &[u8] = &0xff99a589918191ffu64.to_be_bytes();
const GLYPHS: &[(char, &[u8])] = &[
    (' ', &0x0000000000000000u64.to_be_bytes()),
    ('A', &0x1824427e42420000u64.to_be_bytes()),
    ('B', &0x7c427c42427c0000u64.to_be_bytes()),
    ('C', &0x3c424040423c0000u64.to_be_bytes()),
    ('D', &0x7c424242427c0000u64.to_be_bytes()),
    ('E', &0x7e407840407e0000u64.to_be_bytes()),
    ('F', &0x7e40784040400000u64.to_be_bytes()),
    ('G', &0x3c42404e423c0000u64.to_be_bytes()),
    ('H', &0x42427e4242420000u64.to_be_bytes()),
    ('I', &0x3810101010380000u64.to_be_bytes()),
    ('J', &0x0e04040444380000u64.to_be_bytes()),
    ('K', &0x4850605048440000u64.to_be_bytes()),
    ('L', &0x40404040407c0000u64.to_be_bytes()),
    ('M', &0x4266565a4a420000u64.to_be_bytes()),
    ('N', &0x4262524a46420000u64.to_be_bytes()),
    ('O', &0x3c424242423c0000u64.to_be_bytes()),
    ('P', &0x7c42427c40400000u64.to_be_bytes()),
    ('Q', &0x3c4242424a3c0200u64.to_be_bytes()),
    ('R', &0x7c42427c48460000u64.to_be_bytes()),
    ('S', &0x1c22300c44380000u64.to_be_bytes()),
    ('T', &0x7c10101010100000u64.to_be_bytes()),
    ('U', &0x42424242423c0000u64.to_be_bytes()),
    ('V', &0x4444282810100000u64.to_be_bytes()),
    ('W', &0x52525a2c24240000u64.to_be_bytes()),
    ('X', &0x4224181824420000u64.to_be_bytes()),
    ('Y', &0x4428101010100000u64.to_be_bytes()),
    ('Z', &0x7e040810207e0000u64.to_be_bytes()),
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
    ('0', &0x3c464a52623c0000u64.to_be_bytes()),
    ('1', &0x1030101010380000u64.to_be_bytes()),
    ('2', &0x3c42020c307e0000u64.to_be_bytes()),
    ('3', &0x3c420c02423c0000u64.to_be_bytes()),
    ('4', &0x4848487c08080000u64.to_be_bytes()),
    ('5', &0x3e203c02221c0000u64.to_be_bytes()),
    ('6', &0x3c40407e423c0000u64.to_be_bytes()),
    ('7', &0x7e02040810100000u64.to_be_bytes()),
    ('8', &0x3844384444380000u64.to_be_bytes()),
    ('9', &0x3c42423e023c0000u64.to_be_bytes()),
];
