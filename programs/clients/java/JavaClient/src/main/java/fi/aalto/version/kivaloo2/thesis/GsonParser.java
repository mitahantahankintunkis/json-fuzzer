package fi.aalto.version.kivaloo2.thesis;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.nio.charset.StandardCharsets;


public class GsonParser implements Parser {
    private static final String KEY_NOT_FOUND = "KEY_NOT_FOUND";
    private static final String PARSE_ERROR = "PARSE_ERROR";

	GsonParser() {
	}

	@Override
	public String parse(byte[] json, int offset, int len, String key) {
        try {
			String s = new String(json, offset, len, StandardCharsets.UTF_8);
			JsonObject jsonObject = JsonParser.parseString(s).getAsJsonObject();

			if (!jsonObject.has(key)) {
				return this.KEY_NOT_FOUND;
			}

			return jsonObject.getAsJsonPrimitive(key).getAsString();

        } catch (Exception e) {
			return this.PARSE_ERROR;
		}
	}

	@Override
	public String getName() {
		return "java_gson";
	}
}

