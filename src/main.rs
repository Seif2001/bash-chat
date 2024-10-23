use steganography::util::*;
pub mod image_processor;
use std::io::Write;

fn main() {

    // // Load the default image (used as the key)
    // let default_image = file_as_dynamic_image("default.png".to_string());

    // Paths for input and output images
    let input_image_path = "test.png".to_string();
    let encoded_image_path = "test_enc.png".to_string();
    let encoded_image_path_new = "test_enc_new.png".to_string();

    // Encode the image using the payload and the key image
    image_processor::encode_image(input_image_path.clone(), encoded_image_path.clone(), "default1.png".to_string());

    // Decode the image using the encoded image and the key image
    image_processor::decode_image(encoded_image_path.clone(), encoded_image_path_new.clone());
}
