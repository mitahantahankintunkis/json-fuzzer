# Vulnerability report for Mongoose web server

Due to a slow implementation of scientific notation parsing, the function `mg_atod` can be used as a vector for Denial of Service.

`mg_atod` is used by `mg_json_get`, which the following functions depend on:
- mg_json_get
- mg_json_get_b64
- mg_json_get_bool
- mg_json_get_hex
- mg_json_get_num
- mg_json_get_long
- mg_json_get_str
- mg_json_get_tok
- mg_json_next
- mg_rpc_process
- mg_rpc_verr
- mg_rpc_vok
- mg_rpc_ok
- mg_rpc_err
- mg_rpc_call
- mg_rpc_list

Thus, all systems using any of these functions on untrusted data is vulnerable to this.

`mg_rpc_process` is especially vulnerable to this. Having a one megabyte array of form `[9E-3079,9E-3079,...,9E-3079]` in the `id` field takes around 13 seconds to process.

