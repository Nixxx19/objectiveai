use super::*;
use indexmap::IndexMap;

#[test]
fn modalities_plain_string_schema() {
    let schema = InputSchema::String(StringInputSchema { description: None, r#enum: None });
    assert_eq!(schema.modalities(), Modalities::default());
}

#[test]
fn modalities_object_with_image_and_audio() {
    let schema = InputSchema::Object(ObjectInputSchema {
        description: None,
        properties: {
            let mut m = IndexMap::new();
            m.insert("photo".into(), InputSchema::Image(ImageInputSchema { description: None }));
            m.insert("clip".into(), InputSchema::Audio(AudioInputSchema { description: None }));
            m.insert("name".into(), InputSchema::String(StringInputSchema { description: None, r#enum: None }));
            m
        },
        required: None,
    });
    assert_eq!(schema.modalities(), Modalities { image: true, audio: true, video: false, file: false });
}

#[test]
fn modalities_anyof_unions_variants() {
    let schema = InputSchema::AnyOf(AnyOfInputSchema {
        any_of: vec![
            InputSchema::Video(VideoInputSchema { description: None }),
            InputSchema::File(FileInputSchema { description: None }),
            InputSchema::String(StringInputSchema { description: None, r#enum: None }),
        ],
    });
    assert_eq!(schema.modalities(), Modalities { image: false, audio: false, video: true, file: true });
}

#[test]
fn modalities_nested_array_of_anyof() {
    let schema = InputSchema::Array(ArrayInputSchema {
        description: None,
        min_items: Some(1),
        max_items: Some(10),
        items: Box::new(InputSchema::AnyOf(AnyOfInputSchema {
            any_of: vec![
                InputSchema::Image(ImageInputSchema { description: None }),
                InputSchema::Object(ObjectInputSchema {
                    description: None,
                    properties: {
                        let mut m = IndexMap::new();
                        m.insert("recording".into(), InputSchema::Audio(AudioInputSchema { description: None }));
                        m
                    },
                    required: None,
                }),
            ],
        })),
    });
    assert_eq!(schema.modalities(), Modalities { image: true, audio: true, video: false, file: false });
}

#[test]
fn modalities_all_four_media_types() {
    let schema = InputSchema::Object(ObjectInputSchema {
        description: None,
        properties: {
            let mut m = IndexMap::new();
            m.insert("img".into(), InputSchema::Image(ImageInputSchema { description: None }));
            m.insert("aud".into(), InputSchema::Audio(AudioInputSchema { description: None }));
            m.insert("vid".into(), InputSchema::Video(VideoInputSchema { description: None }));
            m.insert("doc".into(), InputSchema::File(FileInputSchema { description: None }));
            m
        },
        required: None,
    });
    assert_eq!(schema.modalities(), Modalities { image: true, audio: true, video: true, file: true });
}
