intervalsets-book
=================

Higher level guides, tutorials and reference material.

This uses mdbook to organize markdown documents in the "content" folder.

## testing

mdbook's test functionality leaves something to be desired. As a work around,
this is also a rust library crate which inlines each markdown file
as a module and runs them as doctests. Each new markdown document added to the
book will need to be manually added as a new module to src/libs.rs to ensure
test coverage.