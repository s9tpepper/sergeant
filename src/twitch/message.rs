#[allow(dead_code)]
struct Tag {
    name: String,
    value: String,
}

#[allow(dead_code)]
struct Message {
    tags: Vec<Tag>,
    prefix: String,
    command: String,
    message: String,
}
