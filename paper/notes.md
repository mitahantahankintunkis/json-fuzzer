# Notes

Some general thoughts I've had.


## Automatic test case generation

In the table below, rows three, four, five, and six contain interesting behavior. The analyzer could detect these and pass them back to the fuzzer.

|Row|JSON|Parser outputs (parser0, parser1, parser2)|
|-|-|-|
|...|...|...|
|0|`{"q":2, "\xfbq":3}`|(`"PARSE_ERROR"`, `2`, `2`)|
|1|`{"q":2, "\xfcq":3}`|(`"PARSE_ERROR"`, `2`, `2`)|
|2|`{"q":2, "\xfdq":3}`|(`"PARSE_ERROR"`, `2`, `2`)|
|3|`{"q":2, "\xfeq":3}`|(`2`, `2`, `2`)|
|4|`{"q":2, "\xffq":3}`|(`2`, `2`, `3`)|
|5|`{"q":2, "q\x00":3}`|(`"PARSE_ERROR"`, `PARSE_ERROR`, `PARSE_ERROR`)|
|6|`{"q":2, "q\x01":3}`|(`"PARSE_ERROR"`, `3`, `PARSE_ERROR`)|
|7|`{"q":2, "q\x02":3}`|(`"PARSE_ERROR"`, `3`, `PARSE_ERROR`)|
|8|`{"q":2, "q\x03":3}`|(`"PARSE_ERROR"`, `3`, `PARSE_ERROR`)|
|9|`{"q":2, "q\x04":3}`|(`"PARSE_ERROR"`, `3`, `PARSE_ERROR`)|
|...|...|...|

