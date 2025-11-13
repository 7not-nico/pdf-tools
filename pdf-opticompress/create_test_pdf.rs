use lopdf::{Document, Object, Stream};
use std::fs;

fn main() {
    // Create a simple PDF for testing
    let mut doc = Document::new();

    // Create a simple page
    let page_id = doc.new_object_id();
    let content = b"BT\n/F1 12 Tf\n100 700 Td\n(Hello, World!) Tj\nET";

    let page = Object::Dictionary(
        lopdf::dictionary!(
            "Type" => "Page",
            "Parent" => 1,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => doc.new_object_id(),
            "Resources" => Object::Dictionary(
                lopdf::dictionary!(
                    "Font" => Object::Dictionary(
                        lopdf::dictionary!(
                            "F1" => Object::Dictionary(
                                lopdf::dictionary!(
                                    "Type" => "Font",
                                    "Subtype" => "Type1",
                                    "BaseFont" => "Helvetica"
                                )
                            )
                        )
                    )
                )
            )
        )
    );

    let content_stream = Object::Stream(Stream::new(
        lopdf::dictionary!(),
        content.to_vec(),
    ));

    doc.objects.insert(page_id, page);
    doc.objects.insert(doc.new_object_id(), content_stream);

    // Create pages array
    let pages_id = doc.new_object_id();
    let pages = Object::Dictionary(
        lopdf::dictionary!(
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1
        )
    );
    doc.objects.insert(pages_id, pages);

    // Create catalog
    let catalog_id = doc.new_object_id();
    let catalog = Object::Dictionary(
        lopdf::dictionary!(
            "Type" => "Catalog",
            "Pages" => pages_id
        )
    );
    doc.objects.insert(catalog_id, catalog);

    // Set trailer
    doc.trailer.set("Root", catalog_id);

    // Save the PDF
    doc.save("test.pdf").unwrap();
    println!("Created test.pdf for testing");
}