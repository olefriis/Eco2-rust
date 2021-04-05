/**
 * Big, messy implementation with chunks copied from XXTEA-Rust (https://github.com/Hanaasagi/XXTEA-Rust).
 * License file is provided separately in this repository.
 */
pub fn decrypt(secret: &Vec<u8>, value: &Vec<u8>) -> Vec<u8> {
  let key_as_u32s = to_u32_vec(secret);
  let mut value_as_u32s: Vec<u32> = to_u32_vec(&switch_endianness(&value));

  let decrypted_value_as_u32s = decrypt_(&mut value_as_u32s, &key_as_u32s);
  switch_endianness(&to_u8_vec(&decrypted_value_as_u32s))
}

pub fn encrypt(secret: &Vec<u8>, value: &Vec<u8>) -> Vec<u8> {
  let key_as_u32s = to_u32_vec(secret);
  let mut value_as_u32s: Vec<u32> = to_u32_vec(&switch_endianness(&value));

  let encrypted_value_as_u32s = encrypt_(&mut value_as_u32s, &key_as_u32s);
  switch_endianness(&to_u8_vec(&encrypted_value_as_u32s))
}

fn switch_endianness(bytes: &Vec<u8>) -> Vec<u8> {
    let mut output = Vec::new();

    for i in 0..bytes.len() {
        let word_offset = (i / 4) * 4;
        let offset_within_word = 3 - i % 4;
        output.push(bytes[word_offset + offset_within_word]);
    }

    output
}

fn to_u32_vec(data: &Vec<u8>) -> Vec<u32> {
    let length = data.len() as u32;

    let n = if (length & 3) == 0 { length >> 2 } else { (length >> 2) + 1 };
    let mut result: Vec<u32> = Vec::new();
    for _ in 0..n {
        result.push(0);
    }

    for i in 0..(length as usize) {
        result[i >> 2] |= (data[i] as u32) << ((i & 3) << 3);
    }
    result
}

fn to_u8_vec(data: &Vec<u32>) -> Vec<u8> {
    let n = data.len() << 2;
    let mut result: Vec<u8> = Vec::new();
    for _ in 0..n {
        result.push(0);
    }

    for i in 0..n {
        result[i] = (data[i >> 2] >> ((i & 3) << 3)) as u8;
    }
    result
}

const DELTA: u32 = 0x9E3779B9;

fn mx(sum: u32, y: u32, z: u32, p: u32, e: u32, k: &Vec<u32>) -> u32 {
    ((z >> 5 ^ y << 2).wrapping_add(y >> 3 ^ z << 4)) ^ ((sum ^ y).wrapping_add(k[(p & 3 ^ e) as usize] ^ z))
}

fn decrypt_(value: &mut Vec<u32>, key: &Vec<u32>) -> Vec<u32> {
    if key.len() != 4 {
        panic!("XXTEA encryption key should be 8 32-bit numbers, was {}", key.len());
    }

    let mut v = value.clone();
    let length: u32 = v.len() as u32;
    let n = length - 1;
    let mut e: u32;
    let mut y = v[0];
    let mut z;
    let q: u32 = 6 + 52 / length;
    let mut sum: u32 = q.wrapping_mul(DELTA);
    while sum != 0 {
        e = sum >> 2 & 3;
        let mut p = n as usize;
        while p > 0 {
            z = v[p - 1];
            v[p] = v[p].wrapping_sub(mx(sum, y, z, p as u32, e, &key));
            y = v[p];
            p = p - 1;
        }
        z = v[n as usize];
        v[0] = v[0].wrapping_sub(mx(sum, y, z, 0, e, &key));
        y = v[0];
        sum = sum.wrapping_sub(DELTA);
    }

    v
}

fn encrypt_(v: &mut Vec<u32>, key: &Vec<u32>) -> Vec<u32> {
  if key.len() != 4 {
    panic!("XXTEA encryption key should be 8 32-bit numbers, was {}", key.len());
  }

  let length: u32 = v.len() as u32;
  let n: u32 = length - 1;
  let mut e: u32;
  let mut y: u32;
  let mut z = v[n as usize];
  let mut sum: u32 = 0;
  let mut q: u32 = 6 + 52 / length;
  while q > 0 {
      sum = sum.wrapping_add(DELTA);
      e = sum >> 2 & 3;
      for p in 0..n {
          y = v[(p as usize) +1];
          v[p as usize] = v[p as usize].wrapping_add(mx(sum, y, z, p as u32, e, &key));
          z = v[p as usize];
      }
      y = v[0];
      v[n as usize] = v[n as usize].wrapping_add(mx(sum, y, z, n, e, &key));
      z = v[n as usize];
      q = q - 1;
  }
  return v.clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_decrypt_a_name() {
      let encrypted_name: Vec<u8> = vec![177, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250];
      let secret: Vec<u8> = vec![215, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65];

      let decrypted_name = decrypt(&secret, &encrypted_name);
      assert_eq!(decrypted_name, vec![65, 108, 114, 117, 109, 32, 111, 112, 103, 97, 110, 103, 0, 0, 0, 0])
    }


    #[test]
    fn it_can_encrypt_a_name() {
      let decrypted_name = vec![65, 108, 114, 117, 109, 32, 111, 112, 103, 97, 110, 103, 0, 0, 0, 0];
      let secret: Vec<u8> = vec![215, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65];

      let encrypted_name = encrypt(&secret, &decrypted_name);
      assert_eq!(encrypted_name, vec![177, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250])
    }
}