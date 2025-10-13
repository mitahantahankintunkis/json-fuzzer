# Master's Thesis Proposal (draft)

## Cross-language JSON parser interoperability

**Student: Oula Kivalo**

**Supervisor: _TODO_**


## Aim and objectives
*Note: While this proposal focuses on JSON parsers, the subject may be changed to other data formats or protocols at the supervisor's discretion.*

JavaScript Object Notation(JSON) has become the de facto standard for data interchange on the internet, and as such, multiple parsers have been developed for it across programming languages. With the rise of microservice-based cloud architectures, where small independent services often utilize JSON for communication, these different parser implementations may be used to parse the same inputs. If a malicious actor is able to inject JSON data into such communication, inconsistencies in parser behavior may enable privilege escalation, denial-of-service attacks, or other vulnerabilities\[4].

Even with its minimalistic design, JSON parsing can introduce security vulnerabilities due to ambiguities in the specification and incorrect parser implementations\[5]. Due to its apparent simplicity and minimalistic featureset, JSON can easily be missed as a potential attack vector.

Despite the widespread use of JSON, academic research on JSON parser interoperability is surprisingly limited\[1]. Most prior work has been carried out by independent security researchers, though their findings are already several years old and may be outdated\[2]\[3]. This thesis aims to further research in the area by documenting interoperability concerns across different parser implementations in multiple programming languages.


## Goals and contributions
The main goal of the thesis is to document discrepancies between parsing results that may lead to vulnerabilities. These results will be compiled into a compatibility matrix showing which parsers should not be used together. An example of this matrix can be found in [Preliminary results](#preliminary-results).

Secondary goal of the thesis is to search for vulnerabilities in JSON parsers and other open source projects, although no major results are guaranteed for this. Examples of potential vulnerabilities can be found at [/poc/proxy_rate_limiting_bypass/](./poc/proxy_rate_limiting_bypass/) and [/poc/API_gateway_schema_bypass/](./poc/API_gateway_schema_bypass/).

The aim is to test at least 40 parsers across 7 to 10 languages which will be selected from [json.org](https://json.org) and GitHub based on their popularity. The research may also be expanded to include JSON parsers in SQL database engines such as SQLite and PostgreSQL.


## Methodology
The thesis will focus on testing how parsers handle different character encodings and duplicate keys in JSON objects. Research will be conducted using a custom fuzzing solution, which mutates different test cases selected from test suites of popular parsers and edge cases identified by the JSON specification\[5]. These mutated test cases will be parsed with different parsers, whose results and execution times are stored for future analyzing.

Using the results gained from the fuzzer, open source projects are manually investigated for potential vulnerabilities stemming from JSON parsing.


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

* **Fuzzer**: A Rust-based application that generates test cases and distributes them to parser clients via TCP sockets. Test cases are based on templates defined in [/programs/payloads.toml](./programs/payloads.toml) and mutated according to configuration parameters.\
The fuzzer can be further optimized and needs new features for more complex test cases.

* **Clients**: Each language has its own client which includes multiple parsers.\
Currently, seven clients are implemented for Rust, Go, Python, Lua, C/C++, Clojure/Java, and PHP, containing a total of 26 different parsers.

* **Analyzer**: A naive and limited implementation written in Rust that compares results produced by different parsers for each test case and saves interesting ones in a `.csv` file for manual review.\
Currently, the analyzer only detects the most obvious cases where two parsers produce different outputs for the same input.

To run the project in its current state, install docker and run the following command:

```
./run-docker.sh
```

Once inside the image, run the following commands:

```
./run.sh
# Stop run.sh after it has finished with <C-c>
./analyze.sh
```

### Preliminary results
The project currently focuses only on investigating how different parsers handle JSON objects containing duplicate keys. The latest JSON specification does not specify the expected behavior for duplicate keys, leaving it to be implementation specific\[5]. This ambiguity has led to vulnerabilities before, for example in Apache CouchDB, where inconsistencies between Erlang and Javascript JSON parsers enabled an attacker to potentially gain administrative access to any public-facing CouchDB instance\[4].

The project has found 654 parser combinations out of 1089 possible (60.1%) where parsers return different values in JSON objects when querying for the same key. These cases are listed in the figure below and the table located at [/analyzed/README.md](./analyzed/README.md).

![JSON parsing mismatches](paper/figures/parsing_mismatches.png "JSON parsing mismatches")

*Figure 1: Parsing mismatches. A cell is colored in blue if the parsers in the corresponding column and row can retrieve different values when querying the same key in the same JSON object.*

### Paper
A preliminary version of the paper can be found at [/paper/thesis.pdf](./paper/thesis.pdf). It only contains some of my initial thoughts on the subject and will be rewritten later, but the general structure is there.


## References
\[1] [https://dl.acm.org/doi/abs/10.1145/3634737.3657003](https://dl.acm.org/doi/abs/10.1145/3634737.3657003)

\[2] [https://bishopfox.com/blog/json-interoperability-vulnerabilities](https://bishopfox.com/blog/json-interoperability-vulnerabilities)

\[3] [https://seriot.ch/projects/parsing\_json.html](https://seriot.ch/projects/parsing_json.html)

\[4] [https://justi.cz/security/2017/11/14/couchdb-rce-npm.html](https://justi.cz/security/2017/11/14/couchdb-rce-npm.html)

\[5] [https://datatracker.ietf.org/doc/html/rfc825](https://datatracker.ietf.org/doc/html/rfc825)

