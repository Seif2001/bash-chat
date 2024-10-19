extern crate steganography;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;

pub fn encode_image(path_input: String, path_output: String, payload: String) {
    let message = payload;
    let payload = str_to_bytes(&message);
    let destination_image = file_as_dynamic_image(path_input);
    let enc = Encoder::new(payload, destination_image);
    let result = enc.encode_alpha();
    save_image_buffer(result, path_output);
}

pub fn decode_image(path_input: String) {
    let encoded_image = file_as_image_buffer(path_input);
    let dec = Decoder::new(encoded_image);
    let out_buffer = dec.decode_alpha();
    let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
    let message = bytes_to_str(clean_buffer.as_slice());
    println!("The message is {:?}", message);
}
