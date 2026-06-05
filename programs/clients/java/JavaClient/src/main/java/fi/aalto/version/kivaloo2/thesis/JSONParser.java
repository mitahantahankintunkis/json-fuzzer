package fi.aalto.version.kivaloo2.thesis;

import org.json.JSONObject;
import org.json.JSONException;
import java.nio.charset.StandardCharsets;


public class JSONParser implements Parser {
    private static final String KEY_NOT_FOUND = "KEY_NOT_FOUND";
    private static final String PARSE_ERROR = "PARSE_ERROR";

	JSONParser() {
	}

	@Override
	public String parse(byte[] json, int offset, int len, String key) {
        try {
			String s = new String(json, offset, len, StandardCharsets.UTF_8);
			JSONObject obj = new JSONObject(s);

			return obj.optString(key, this.KEY_NOT_FOUND).toString();
        } catch (Exception e) {
			return this.PARSE_ERROR;
		}
	}

	@Override
	public String getName() {
		return "java_json";
	}
}

