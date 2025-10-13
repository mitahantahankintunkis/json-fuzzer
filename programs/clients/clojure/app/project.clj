(defproject app "0.1.0-SNAPSHOT"
  :description "FIXME: write description"
  :url "http://example.com/FIXME"
  :license {:name "EPL-2.0 OR GPL-2.0-or-later WITH Classpath-exception-2.0"
            :url "https://www.eclipse.org/legal/epl-2.0/"}
  :dependencies [[org.clojure/clojure "1.10.0"]
                 [metosin/jsonista "0.3.13"]
                 [com.fasterxml.jackson.core/jackson-databind "2.20.0"]
                 [cheshire "6.1.0"]
                 [com.google.code.gson/gson "2.13.2"]]
  :main ^:skip-aot app.core
  :target-path "target/%s"
  :profiles {:uberjar {:aot :all}})
