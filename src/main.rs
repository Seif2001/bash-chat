pub mod image_processor;
fn main() {
    image_processor::encode_image(
        "penguin.png".to_string(),
        "penguin_enc.png".to_string(),
        "{views:10}".to_string(),
    );
    image_processor::decode_image("penguin_enc.png".to_string());
}
