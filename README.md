# Aide Comment

This crate provides a macro that can be used to extract a summary and
description for an OpenAPI operation from doc comments. This crate supports
[axum](https://crates.io/crates/axum) and integrates this information with
[aide](https://crates.io/crates/aide).

```
/// This is a summary
///
/// This is a longer description of the endpoint that is expected to be much
/// more detailed and may span more lines than the first paragraph summary.
#[aidecomment]
async fn my_handler() -> &'static str {
    "hello world"
}
```
