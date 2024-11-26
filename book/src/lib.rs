mod introduction {
    doc_comment::doctest!("../content/introduction.md");
}

mod quickstart {
    mod api {
        doc_comment::doctest!("../content/quickstart/api.md");
    }
    mod examples {
        doc_comment::doctest!("../content/quickstart/example.md");
    }
}

mod design {
    mod goals {
        doc_comment::doctest!("../content/design/goals.md");
    }
    mod types {
        doc_comment::doctest!("../content/design/types.md");
    }
    mod errors {
        doc_comment::doctest!("../content/design/errors.md");
    }
    mod footguns {
        doc_comment::doctest!("../content/design/footguns.md");
    }
}
