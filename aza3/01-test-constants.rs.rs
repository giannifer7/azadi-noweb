// Test Constants and Sample Data
// All the test fixtures used across different test cases

const BASIC_CHUNK: &str = r#"
# <<test>>=
Hello
# @
"#;

const TWO_CHUNKS: &str = r#"
# <<chunk1>>=
First chunk
# @
# <<chunk2>>=
Second chunk
# @
"#;

const NESTED_CHUNKS: &str = r#"
# <<outer>>=
Before
# <<inner>>
After
# @
# <<inner>>=
Nested content
# @
"#;

const INDENTED_CHUNK: &str = r#"
# <<main>>=
    # <<indented>>
# @
# <<indented>>=
some code
# @
"#;

const PYTHON_CODE: &str = r#"
# <<code>>=
def example():
    # <<body>>
# @
# <<body>>=
print('hello')
# @
"#;

const MULTI_COMMENT_CHUNKS: &str = r#"
# <<python_chunk>>=
def hello():
    print("Hello")
# @

// <<rust_chunk>>=
fn main() {
    println!("Hello");
}
// @
"#;

const FILE_CHUNKS: &str = r#"
# <<@file output.txt>>=
content
# @
# <<other>>=
other content
# @
"#;

const TWO_FILES: &str = r#"
# <<@file file1.txt>>=
Content 1
# @
# <<@file file2.txt>>=
Content 2
# @
"#;

const SEQUENTIAL_CHUNKS: &str = r#"
# <<main>>=
# <<part1>>
# <<part2>>
# @
# <<part1>>=
First part
# @
# <<part2>>=
Second part
# @
"#;

const EMPTY_CHUNK: &str = r#"
# <<empty>>=
# @
"#;