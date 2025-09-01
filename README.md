# Master's Thesis Proposal (draft)

## Cross-language parser interoperability

**Student: Oula Kivalo**

**Supervisor: _TODO_**


## Aim and objectives
*Note: While this proposal focuses on JSON parsers, the subject may be changed to other data formats or protocols at the supervisor's discretion.*

JavaScript Object Notation(JSON) has become the de facto standard for data interchange on the internet, and as such, multiple parsers have been developed for it across programming languages. With the rise of microservice-based cloud architectures, where small independent services often utilize JSON for communication, these different parser implementations may be used to parse the same inputs. If a malicious actor is able to inject JSON data into such communication, inconsistencies in parser behavior may enable privilege escalation, denial-of-service attacks, or other vulnerabilities\[4].

Even with its minimalistic design, JSON parsing can introduce security vulnerabilities due to ambiguities in the specification and incorrect parser implementations\[5]. Due to its apparent simplicity and minimalistic featureset, JSON can easily be missed as a potential attack vector.

Despite the widespread use of JSON, academic research on JSON parser interoperability is surprisingly limited\[1]. Most prior work has been carried out by independent security researchers, though their findings are already several years old and may be outdated\[2]\[3]. This thesis aims to further research in the area by documenting interoperability concerns across different parser implementations in multiple programming languages.


## Goals and contributions
The main goal of the thesis is to document discrepancies between parsing results that may lead to vulnerabilities. These results will be compiled into a compatibility matrix showing which parsers should not be used together. An example of this matrix can be found in [Preliminary results](#preliminary-results). A secondary goal is to search for vulnerabilities in individual parsers, although no results are guaranteed for this.

The aim is to test at least 40 parsers across 7 to 10 languages which will be selected from [json.org](https://json.org) and GitHub based on their popularity. The research may also be expanded to include JSON parsers in SQL database engines such as SQLite and PostgreSQL.


## Methodology
The thesis will focus on testing how parsers handle different character encodings and duplicate keys in JSON objects. Research will be conducted using a custom fuzzing solution, which mutates different test cases selected from test suites of popular parsers and edge cases identified by the JSON specification\[5]. These mutated test cases will be parsed with different parsers, whose results and execution times are stored for future analyzing.


## Schedule
I am aiming to complete the thesis between December and March. I have completed my studies, with the exception of two study points, and I am not working at the moment. Thus, I am able to put my full focus on the thesis.

|Task|Duration|
|----|--------|
|Research|1-3 weeks|
|Programming|3-4 weeks|
|Fuzzing and analyzing results|4-5 weeks|
|Writing the paper|6-10 weeks|

*Table 1: Rough time estimates for different tasks in the thesis. Some tasks may have overlap between each other.*


## Work completed so far
This project includes a proof-of-concept fuzzing application capable of testing tens of millions of test cases per second across different languages and parsers.

The project currently consists of three main components:

* **Fuzzer**: A Rust-based application that generates test cases and distributes them to parser clients via TCP sockets. Test cases are based on templates defined in [./payloads.toml](payloads.toml) and mutated according to configuration parameters.\
The fuzzer can be further optimized and needs new features for more complex test cases.

* **Clients**: Each language has its own client that includes multiple parsers.\
Currently, four clients are implemented for Rust, Go, Python, and C, containing a total of 19 different parsers.

* **Analyzer**: A naive and limited implementation written in Rust that compares results produced by different parsers for each test case and saves interesting ones in a `.csv` file for manual review.\
Currently, the analyzer only detects the most obvious cases where two parsers produce different outputs for the same input.


### Preliminary results
The project currently focuses only on investigating how different parsers handle JSON objects containing duplicate keys. The latest JSON specification does not specify the expected behavior for duplicate keys, leaving it to be implementation specific\[5]. This ambiguity has led to vulnerabilities before, for example in Apache CouchDB, where inconsistencies between Erlang and Javascript JSON parsers enabled an attacker to potentially gain administrative access to any public-facing CouchDB instance\[4].

The project has found 226 parser combinations out of 361 possible (63%) where parsers return different values in JSON objects when querying for the same key. These cases are listed in the figure and table below.

![JSON parsing mismatches](paper/figures/parsing_mismatches.png "JSON parsing mismatches")

*Figure 1: Parsing mismatches. A cell is colored in blue if the parsers in the corresponding column and row can retrieve different values when querying the same key in the same JSON object.*

|P1|P2|JSON|P1\["q"]|P2\["q"]|
|-------|-------|----|:-----:|:-----:|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[c_jansson](https://github.com/akheron/jansson)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q\u0000":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q\u0000":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q\u0000":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"\u0071":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"q":3}|2|3|
|[c_cjson](https://github.com/DaveGamble/cJSON)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"q":3}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":3,"q":2}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[c_jansson](https://github.com/akheron/jansson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[c_jansson](https://github.com/akheron/jansson)|{"q":2," ":3}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":3,"q":2}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_orjson](https://github.com/ijl/orjson)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2," ":3}|3|2|
|[c_json_parser](https://github.com/json-parser/json-parser)|[rust_serde](https://github.com/serde-rs/json)|{"q":2," ":3}|3|2|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q\u0000":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[c_jansson](https://github.com/akheron/jansson)|{"q":3,"q":2}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":3,"q":2}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q" 2,"q":3}|3|2|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"\u0071":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"q":3}|2|3|
|[go_buger_jsonparser](https://github.com/buger/jsonparser)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"q":3}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[c_jansson](https://github.com/akheron/jansson)|{"q":2,"Q":3}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":2,"Q":3}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"Q":3}|3|2|
|[go_json_iterator](https://github.com/json-iterator/go)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[c_jansson](https://github.com/akheron/jansson)|{"q":2,"Q":3}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":2,"Q":3}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[go_std](https://pkg.go.dev/encoding/json)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"Q":3}|3|2|
|[go_std](https://pkg.go.dev/encoding/json)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"Q":3}|3|2|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":3,"q":2}|2|3|
|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q\u0000":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[c_jansson](https://github.com/akheron/jansson)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q" 2,"q":3}|3|2|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q\q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson](https://github.com/tidwall/gjson)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q\u0000":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[c_jansson](https://github.com/akheron/jansson)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":3,"q":2}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"\u0071":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"q":3}|2|3|
|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"\u0071":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[c_jansson](https://github.com/akheron/jansson)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"\u0071":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_francoispqt_gojay](https://github.com/francoispqt/gojay)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_sugawarayuuta_sonnet](https://github.com/sugawarayuuta/sonnet)|{"q":3,"q":2}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q\q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"\u0071":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_json](https://docs.python.org/3/library/json.html)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_msgspec](https://github.com/jcrist/msgspec)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_orjson](https://github.com/ijl/orjson)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_simplejson](https://github.com/simplejson/simplejson)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[python_ujson](https://github.com/ultrajson/ultrajson)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[rust_json](https://github.com/maciejhirsz/json-rust)|{"q":2,"q":3}|2|3|
|[go_valyala_fastjson](https://github.com/valyala/fastjson)|[rust_serde](https://github.com/serde-rs/json)|{"q":2,"q":3}|2|3|
|[python_json](https://docs.python.org/3/library/json.html)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_json](https://docs.python.org/3/library/json.html)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_json](https://docs.python.org/3/library/json.html)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_json](https://docs.python.org/3/library/json.html)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_json](https://docs.python.org/3/library/json.html)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_json](https://docs.python.org/3/library/json.html)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_json](https://docs.python.org/3/library/json.html)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_json](https://docs.python.org/3/library/json.html)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[python_msgspec](https://github.com/jcrist/msgspec)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_msgspec](https://github.com/jcrist/msgspec)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_msgspec](https://github.com/jcrist/msgspec)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[python_orjson](https://github.com/ijl/orjson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_orjson](https://github.com/ijl/orjson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_orjson](https://github.com/ijl/orjson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_orjson](https://github.com/ijl/orjson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_orjson](https://github.com/ijl/orjson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_orjson](https://github.com/ijl/orjson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_orjson](https://github.com/ijl/orjson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_orjson](https://github.com/ijl/orjson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_rapidjson](https://github.com/python-rapidjson/python-rapidjson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[python_simplejson](https://github.com/simplejson/simplejson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_simplejson](https://github.com/simplejson/simplejson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_simplejson](https://github.com/simplejson/simplejson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[python_ujson](https://github.com/ultrajson/ultrajson)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[rust_json](https://github.com/maciejhirsz/json-rust)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|
|[rust_serde](https://github.com/serde-rs/json)|[c_cjson](https://github.com/DaveGamble/cJSON)|{"q":2,"q":3}|2|3|
|[rust_serde](https://github.com/serde-rs/json)|[c_json_parser](https://github.com/json-parser/json-parser)|{"q":2," ":3}|3|2|
|[rust_serde](https://github.com/serde-rs/json)|[go_buger_jsonparser](https://github.com/buger/jsonparser)|{"q":2,"q":3}|2|3|
|[rust_serde](https://github.com/serde-rs/json)|[go_json_iterator](https://github.com/json-iterator/go)|{"q":2,"Q":3}|3|2|
|[rust_serde](https://github.com/serde-rs/json)|[go_std](https://pkg.go.dev/encoding/json)|{"q":2,"Q":3}|3|2|
|[rust_serde](https://github.com/serde-rs/json)|[go_tidwall_gjson](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[rust_serde](https://github.com/serde-rs/json)|[go_tidwall_gjson_safe](https://github.com/tidwall/gjson)|{"q":2,"q":3}|2|3|
|[rust_serde](https://github.com/serde-rs/json)|[go_valyala_fastjson](https://github.com/valyala/fastjson)|{"q":2,"q":3}|2|3|

*Table 2: Parsing mismatches. Columns P1 and P2 contain different parsers, while columns P1\["q"] and P2\["q"] list values the corresponding parsers retrieve under key 'q'*


## References
\[1] [https://dl.acm.org/doi/abs/10.1145/3634737.3657003](https://dl.acm.org/doi/abs/10.1145/3634737.3657003)

\[2] [https://bishopfox.com/blog/json-interoperability-vulnerabilities](https://bishopfox.com/blog/json-interoperability-vulnerabilities)

\[3] [https://seriot.ch/projects/parsing\_json.html](https://seriot.ch/projects/parsing_json.html)

\[4] [https://justi.cz/security/2017/11/14/couchdb-rce-npm.html](https://justi.cz/security/2017/11/14/couchdb-rce-npm.html)

\[5] [https://datatracker.ietf.org/doc/html/rfc825](https://datatracker.ietf.org/doc/html/rfc825)

