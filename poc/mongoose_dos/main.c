#include "mongoose.h"


static struct mg_rpc *s_rpc_head = NULL;


static void rpc_sum(struct mg_rpc_req *r) {
  double a = 0.0, b = 0.0;
  mg_json_get_num(r->frame, "$.params[0]", &a);
  mg_json_get_num(r->frame, "$.params[1]", &b);
  mg_rpc_ok(r, "%g", a + b);
}


// Contains hypothetical REST API and JSON RPC endpoints.
// REST API located at localhost:8000
// RPC is located at localhost:8000/rpc
static void ev_handler(struct mg_connection *c, int ev, void *ev_data) {
	if (ev == MG_EV_HTTP_MSG) {
		struct mg_http_message *hm = (struct mg_http_message *) ev_data;

		if (mg_match(hm->method, mg_str("POST"), NULL)) {
			if (mg_match(hm->uri, mg_str("/rpc"), NULL)) {
				struct mg_iobuf io = {0, 0, 0, 512};
				// mg_iobuf_resize(&io, hm->body.len + 2);
				mg_iobuf_resize(&io, 1998849);
				int l = io.size;

				struct mg_rpc_req r = {&s_rpc_head, 0, mg_pfn_iobuf, &io, 0, hm->body};
				// struct mg_rpc_req r = {&s_rpc_head, 0, mg_putchar_iobuf_static, &io, 0, hm->body};

				// struct mg_iobuf *io = (struct mg_iobuf *) param;
				// if (expand && io->len + 2 > io->size) mg_iobuf_resize(io, io->len + 2);

				mg_rpc_process(&r);
				if (io.buf) mg_http_reply(c, 200, "", "%s\r\n%d", (char*)io.buf, l);
				else mg_http_reply(c, 400, "", "%m\n", MG_ESC("Error: Invalid RPC call"));
				mg_iobuf_free(&io);

			} else {
				int len;
				mg_json_get(hm->body, "$", &len);
				mg_http_reply(c, 200, "", "{%m:%d}\n", MG_ESC("parsed_len"), len);
			}

		} else {
			mg_http_reply(c, 400, "", "%m\n", MG_ESC("Error: Expected POST request"));
		}
	}
}


int main() {
	struct mg_mgr mgr;
	mg_mgr_init(&mgr);
	mg_http_listen(&mgr, "http://0.0.0.0:8000", ev_handler, NULL);
	mg_rpc_add(&s_rpc_head, mg_str("sum"), rpc_sum, NULL);

	for (;;) {
		mg_mgr_poll(&mgr, 1000);
	}

	mg_mgr_free(&mgr);
	mg_rpc_del(&s_rpc_head, NULL);
	return 0;
}
