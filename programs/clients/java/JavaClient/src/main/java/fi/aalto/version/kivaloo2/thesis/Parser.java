package fi.aalto.version.kivaloo2.thesis;


public interface Parser {
	String parse(byte[] json, int offset, int len, String key);
	String getName();
}
