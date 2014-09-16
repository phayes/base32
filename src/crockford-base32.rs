#![crate_name="crockford-base32"]
#![crate_type="rlib"]

#![feature(phase)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;

pub fn encode(data: &[u8]) -> Vec<Ascii> {
    let alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ".to_ascii();
    let mut ret = Vec::with_capacity((data.len()+3)/4*5);

    for chunk in data.chunks(5) {
        let buf = {
            let mut buf = [0u8, ..5];
            buf.clone_from_slice(chunk);
            buf
        };
        ret.push(alphabet[((buf[0] & 0xF8) >> 3) as uint]);
        ret.push(alphabet[(((buf[0] & 0x07) << 2) | ((buf[1] & 0xC0) >> 6)) as uint]);
        ret.push(alphabet[((buf[1] & 0x3E) >> 1) as uint]);
        ret.push(alphabet[(((buf[1] & 0x01) << 4) | ((buf[2] & 0xF0) >> 4)) as uint]);
        ret.push(alphabet[(((buf[2] & 0x0F) << 1) | (buf[3] >> 7)) as uint]);
        ret.push(alphabet[((buf[3] & 0x7C) >> 2) as uint]);
        ret.push(alphabet[(((buf[3] & 0x03) << 3) | ((buf[4] & 0xE0) >> 5)) as uint]);
        ret.push(alphabet[(buf[4] & 0x1F) as uint]);
    }

    if data.len() % 5 != 0 {
        let len = ret.len();
        ret.truncate(len-8+(data.len()%5*8+4)/5);
    }
                
    ret
}

static inv_alphabet: [u8, ..43] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, 10, 11, 12, 13, 14, 15, 16, 17, 1, 18, 19, 1, 20, 21, 0, 22, 23, 24, 25, 26, -1, 27, 28, 29, 30, 31];

pub fn decode(data: &[Ascii]) -> Option<Vec<u8>> {
    let output_length = data.len()*5/8;
    let mut ret = Vec::with_capacity((output_length+4)/5*5);
    for chunk in data.chunks(8) {
        let buf = {
            let mut buf = [0u8, ..8];
            for (i, &c) in chunk.iter().enumerate() {
                match inv_alphabet.get((c.to_uppercase().to_byte()-('0' as u8)) as uint) {
                    Some(&-1) | None => return None,
                    Some(&value) => buf[i] = value,
                };
            }
            buf
        };
        ret.push((buf[0] << 3) | (buf[1] >> 2));
        ret.push((buf[1] << 6) | (buf[2] << 1) | (buf[3] >> 4));
        ret.push((buf[3] << 4) | (buf[4] >> 1));
        ret.push((buf[4] << 7) | (buf[5] << 2) | (buf[6] >> 3));
        ret.push((buf[6] << 5) | buf[7]);
    }
    ret.truncate(output_length);
    Some(ret)
}

#[cfg(test)]
mod test {
    extern crate test;
    use super::{encode, decode};
    use quickcheck;
    use std;
    use std::rand::distributions::IndependentSample;

    #[deriving(Clone)]
    struct B32 {
        c: Ascii
    }

    impl quickcheck::Arbitrary for B32 {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> B32 {
            let alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ".to_ascii();
            B32 {
                c: alphabet[std::rand::distributions::Range::new(0, alphabet.len()).ind_sample(g)]
            }
        }
    }
    
    impl std::fmt::Show for B32 {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::FormatError> {
            self.c.fmt(f)
        }
    }

    #[test]
    fn masks() {
        assert_eq!(encode([0xF8, 0x3E, 0x0F, 0x83, 0xE0]), Vec::from_slice("Z0Z0Z0Z0".to_ascii()));
        assert_eq!(encode([0x07, 0xC1, 0xF0, 0x7C, 0x1F]), Vec::from_slice("0Z0Z0Z0Z".to_ascii()));
        assert_eq!(decode("Z0Z0Z0Z0".to_ascii()).unwrap(), vec![0xF8, 0x3E, 0x0F, 0x83, 0xE0]);
        assert_eq!(decode("0Z0Z0Z0Z".to_ascii()).unwrap(), vec![0x07, 0xC1, 0xF0, 0x7C, 0x1F]);
    }

    #[quickcheck]
    fn invertible(data: Vec<u8>) -> bool {
        decode(encode(data.as_slice()).as_slice()).unwrap() == data
    }

    #[quickcheck]
    fn lower_case(data: Vec<B32>) -> bool {
        let data: Vec<Ascii> = data.iter().map(|e| e.c).collect();
        decode(data.as_slice()) == decode(data.as_slice().to_lower().as_slice())
    }

    #[test]
    #[allow(non_snake_case)]
    fn iIlL1_oO0() {
        assert_eq!(decode("IiLlOo".to_ascii()), decode("111100".to_ascii()));
    }

    #[test]
    fn invalid_chars() {
        assert_eq!(decode(",".to_ascii()), None)
    }

    #[bench]
    fn bench_encode(b: &mut test::Bencher) {
        let data = [0, 0, 0, 0, 0];
        b.iter(|| encode(data.as_slice()));
        b.bytes = data.len() as u64;
    }

    #[bench]
    fn bench_decode(b: &mut test::Bencher) {
        let data = "00000000".to_ascii();
        b.iter(|| decode(data));
        b.bytes = data.len() as u64;
    }
}
