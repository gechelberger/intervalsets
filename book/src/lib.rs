doc_comment::doctest!("../content/introduction.md", introduction);

mod quickstart {
    doc_comment::doctest!("../content/quickstart/api.md", api);
    doc_comment::doctest!("../content/quickstart/example.md", examples);
}

mod design {
    doc_comment::doctest!("../content/design/goals.md", goals);
    doc_comment::doctest!("../content/design/types.md", types);
    doc_comment::doctest!("../content/design/errors.md", errors);
    doc_comment::doctest!("../content/design/footguns.md", footguns);
}
