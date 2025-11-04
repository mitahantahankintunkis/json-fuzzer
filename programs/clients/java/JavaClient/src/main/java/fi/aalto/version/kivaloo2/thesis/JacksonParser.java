package fi.aalto.version.kivaloo2.thesis;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
// import java.nio.charset.StandardCharsets;

class Query {
	public double q;
}

public class JacksonParser implements Parser {
	private ObjectMapper objectMapper;
    private static final String KEY_NOT_FOUND = "KEY_NOT_FOUND";
    private static final String PARSE_ERROR = "PARSE_ERROR";

	JacksonParser() {
		this.objectMapper = new ObjectMapper();
	}

	@Override
	public String parse(byte[] json, int offset, int len, String key) {
		// String s = new String(json, offset, len, StandardCharsets.UTF_8);

		try {
			if (key.equals("q")) {
				Query q = this.objectMapper.readValue(json, offset, len, Query.class);
				// Query q = this.objectMapper.readValue(s, Query.class);

				if (q.q == (long)q.q) {
					return String.valueOf((long)q.q);
				}

				return String.valueOf(q.q);
			} else {
				JsonNode root = this.objectMapper.readTree(json, offset, len);
				// JsonNode root = this.objectMapper.readTree(s);
				JsonNode el = root.get(key);
				if (el == null) return KEY_NOT_FOUND;

				if (el.isTextual()) {
					return el.asText();
				} else {
					return el.toString();
				}
			}
		} catch (Exception e) {
			return PARSE_ERROR;
		}
	}

	@Override
	public String getName() {
		return "java_jackson";
	}
}

