# Parsing mismatches

The table in [parsing_mismatches.csv](parsing_mismatches.csv) contains JSON objects which are parsed differently between two JSON parsers. These objects can be used to for example bypass input validation as seen in [CVE-2017-12635](https://justi.cz/security/2017/11/14/couchdb-rce-npm.html). The table contains five columns: Parser1, Parser2, JSON, Parser1\["q"], and Parser2\["q"].
