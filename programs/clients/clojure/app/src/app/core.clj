(ns app.core
  (:gen-class)
  (:require (jsonista [core])
            (clojure [string])
            (cheshire [core]))
  (:import (com.fasterxml.jackson.databind ObjectMapper)
           ; (com.google.gson Gson)
           (java.net Socket)
           (java.io DataInputStream DataOutputStream)))

; BufferedOutputStream BufferedInputStream might be more performant

(defn int-to-byte
  [i]
  (byte (if (< i 128)
          i
          (- i 256))))

(defn byte-to-int
  [b]
  (if (< b 0)
    (+ b 256)
    b))

(defn bytes_to_u16
  [byte_array offset]
  (bit-or (byte-to-int (aget byte_array offset))
          (bit-shift-left (byte-to-int (aget byte_array (+ offset 1))) 8)))

(defn bytes_to_u32
  [byte_array offset]
  (bit-or (byte-to-int (aget byte_array offset))
          (bit-shift-left (byte-to-int (aget byte_array (+ offset 1))) 8)
          (bit-shift-left (byte-to-int (aget byte_array (+ offset 2))) 16)
          (bit-shift-left (byte-to-int (aget byte_array (+ offset 3))) 24)))

(def PARSE_ERROR (.getBytes "PARSE_ERROR"))
(def KEY_NOT_FOUND (.getBytes "PARSE_ERROR"))

;; Testing
(defn byte_to_string
  [b]
  (let [i (byte-to-int (int b))]
    (cond
      (or (<= 0x00 i 0x06) (<= 0x0e i 0x1f) (<= 0x7f i 0xff)) (format "<\\x%02x>" i)
      (= 0x07 i) "\\a"
      (= 0x08 i) "\\b"
      (= 0x09 i) "\\t"
      (= 0x0a i) "\\n"
      (= 0x0b i) "\\v"
      (= 0x0c i) "\\f"
      (= 0x0d i) "\\r"
      :else (char i))))

; (def object-mapper jsonista.core/default-object-mapper)

(defn parse_jsonista
  [payload obj_key]
  (try
    (-> payload
        (jsonista.core/read-value)
        (get obj_key KEY_NOT_FOUND)
        (.toString)
        (.getBytes))
    (catch Exception _ PARSE_ERROR)))

(def object-mapper (ObjectMapper.))
(defn parse_jackson
  [payload obj_key]
  (try (-> object-mapper
           (.readTree payload)
           (.get obj_key)
           (.asText)
           (.getBytes))
       (catch NullPointerException _ KEY_NOT_FOUND)
       (catch Exception _ PARSE_ERROR)))

(defn parse_cheshire
  [payload obj_key]
  (try
    (-> payload
        (String.)
        (cheshire.core/parse-string)
        (get obj_key KEY_NOT_FOUND)
        (.toString)
        (.getBytes))
    (catch Exception _ PARSE_ERROR)))

; (def gson (Gson.))
; ; (def maptype (TypeToken.))
; (defn parse_gson
;   [payload obj_key]
;   (gson/fromJson payload))

(defn connect []
  (let [socket (try
                 (let [socket (Socket. "127.0.0.1" 5000)]
                   (.setTcpNoDelay socket true)
                   socket)
                 (catch Exception _ (Thread/sleep 100)))]

    (if (nil? socket)
      (recur)
      socket)))

; (defn u16_to_bytes
;   [byte_array offset value]
;   (aset byte_array offset (bit-and value 0xff))
;   (aset byte_array (+ offset 1) (bit-and (bit-shift-right value 8) 0xff)))
;
; (defn u32_to_bytes
;   [byte_array offset value]
;   (aset byte_array offset (bit-and value 0xff))
;   (aset byte_array (+ offset 1) (bit-and (bit-shift-right value 8) 0xff))
;   (aset byte_array (+ offset 2) (bit-and (bit-shift-right value 16) 0xff))
;   (aset byte_array (+ offset 3) (bit-and (bit-shift-right value 24) 0xff)))

(defn read_all
  [in buffer offset n]
  (loop [total 0]
    (let [len (.read in buffer (+ offset total) (- n total))]
      (cond
        (= len 0) total
        (>= (+ total len) n) (+ total len)
        :else (recur (+ total len))))))

(defn -main
  "JSON fuzzing client for Clojure/Java/Scala"
  [& args]

  ; (time
  ;  (doseq [_ (range 0 1000000)]
  ;    (parse_jsonista (.getBytes "{\"q\":1,\"q\":2}") "q")))
  ;
  ; (time
  ;  (doseq [_ (range 0 1000000)]
  ;    (parse_jsonista (.getBytes "{\"q\":1\"q\":2}") "q")))

; ;; Testing
  ; (let [json (.getBytes "{\"qqq\":0,\"Aqqq\":1234}")]
  ;   (doseq [i (range 0 256)]
  ;     ; (aset-byte json 11 (int-to-byte i))
  ;     (aset-byte json 10 (int-to-byte i))
  ;     (print (map #(format "%02x" %) json) (clojure.string/join (map byte_to_string json)) " -> ")
  ;     (println (clojure.string/join (map byte_to_string (parse_jsonista json "qqq"))))))

  (let [parser_number (Integer/parseInt (or (first args) "0"))
        [parser_name parse_fn] (cond
                                 (= parser_number 0) ["clojure_jsonista" parse_jsonista]
                                 (= parser_number 1) ["java_jackson" parse_jackson]
                                 (= parser_number 2) ["clojure_cheshire" parse_cheshire]
                                 :else (System/exit 1))
        socket (connect)
        ; in (-> socket
        ;        (.getInputStream)
        ;        (BufferedInputStream.)
        ;        (DataInputStream.)
        ;        DataInputStream.)
        ; out (-> socket
        ;         (.getOutputStream)
        ;         (BufferedOutputStream.)
        ;         (DataOutputStream.))
        in (DataInputStream. (.getInputStream socket))
        out (DataOutputStream. (.getOutputStream socket))
        header (make-array Byte/TYPE 10)
        read_buffer (make-array Byte/TYPE (bit-shift-left 1 20))
        write_buffer (make-array Byte/TYPE (bit-shift-left 1 22))]

    ;; Send name
    (.write out (-> parser_name
                    (.getBytes)
                    (java.util.Arrays/copyOf 64)))

    (loop []
      (when (< (.read in header 0 10) 10)
        (System/exit 0))

      (let [_buffer_size (bytes_to_u32 header 0)
            payload_size (bytes_to_u16 header 4)
            batch_size (bytes_to_u32 header 6)]

        (let [l (read_all in read_buffer 0 (* payload_size batch_size))]
          (when (not= l (* payload_size batch_size))
            (println "Read size mismatch" l (* payload_size batch_size))
            (flush)
            (System/exit 1)))

        (let [buf (make-array Byte/TYPE payload_size)
              len (loop [batch 0 written_count 0]
                    (if (< batch batch_size)
                      (do
                        (System/arraycopy read_buffer (* batch payload_size) buf 0 payload_size)

                        ;; Parse JSON in the batch
                        (let [parsed_bytes (parse_fn buf "q")
                              len (count parsed_bytes)]

                          ;; Parsed length
                          (aset-byte write_buffer (+ written_count 4) (int-to-byte (bit-and len 0xff)))
                          (aset-byte write_buffer (+ written_count 5) (int-to-byte (bit-and (bit-shift-right len 8) 0xff)))

                          ;; Parsed bytes
                          (doseq [[j parsed_byte] (map-indexed vector parsed_bytes)]
                            (aset-byte write_buffer (+ written_count j 6) parsed_byte))

                          (recur (inc batch) (+ written_count len 2))))
                      written_count))]

          (aset-byte write_buffer 0 (int-to-byte (bit-and len 0xff)))
          (aset-byte write_buffer 1 (int-to-byte (bit-and (bit-shift-right len 8) 0xff)))
          (aset-byte write_buffer 2 (int-to-byte (bit-and (bit-shift-right len 16) 0xff)))
          (aset-byte write_buffer 3 (int-to-byte (bit-and (bit-shift-right len 24) 0xff)))
          (.write out write_buffer 0 (+ len 4))
          (.flush out))
        (recur)))))

