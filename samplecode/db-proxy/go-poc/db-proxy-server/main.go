package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"

	"github.com/syndtr/goleveldb/leveldb"
)

type request struct {
	ReqType string `json:"req_type"`
	Key     string `json:"key"`
	Value   string `json:"value"`
}

type response struct {
	RspStatus bool   `json:rsp_status`
	Data      string `json:data`
}

func main() {

	fmt.Println("start db-proxy-server")

	db := startleveldb()
	defer db.Close()

	http.HandleFunc("/db-proxy", func(w http.ResponseWriter, r *http.Request) {
		b, err := ioutil.ReadAll(r.Body)
		if err != nil {
			log.Println("Read failed:", err)
		}
		defer r.Body.Close()

		if err != nil {
			log.Println("json format error:", err)
		}
		rsp := forwardServer(db, b)
		if err != nil {
			rsp.RspStatus = false
		}

		rspBytes, _ := json.Marshal(rsp)
		fmt.Fprint(w, string(rspBytes))

	})

	log.Fatal(http.ListenAndServe(":8080", nil))
}

func startleveldb() *leveldb.DB {
	// The returned DB instance is safe for concurrent use. Which mean that all
	// DB's methods may be called concurrently from multiple goroutine.
	db, err := leveldb.OpenFile("./../db", nil)
	if err != nil {
		panic("failed to start the leveldb")
	}
	return db
}

func forwardServer(db *leveldb.DB, reqByte []byte) response {
	rsp:= response{RspStatus:true}
	var err error
	var data []byte

	req := &request{}
	err = json.Unmarshal(reqByte, req)

	switch req.ReqType {
	case "get":
		data, err = db.Get([]byte(req.Key), nil)
		rsp.Data = string(data)
		break
	case "put":
		err = db.Put([]byte(req.Key), []byte(req.Value), nil)
		break
	case "delete":
		err = db.Delete([]byte(req.Key), nil)
		break
	default:
	}

	if err != nil {
		rsp.RspStatus = false
	} else {
		fmt.Printf("request failed:%s\n", string(reqByte))
	}
	return rsp
}
