package main

import (
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
)

type request struct {
	ReqType         string `json:"req_type"`
	Key             string `json:"key"`
	Value           string `json:"value"`
	PresentRootHash string `json:"present_root_hash"`
	DeletedRootHash string `json:"deleted_root_hash"`
}

type response struct {
	RspStatus bool   `json:rsp_status`
	Data      string `json:data`
	ErrorInfo string `json:errorInfo`
}

var mode = flag.String("mode", "start", "start mode")

func main() {

	fmt.Println("start db-client")

	flag.Parse()
	fmt.Println(*mode)
	if *mode == "start" {
		initialData()

		modifyData()

		persistData()
	} else if *mode == "reload" {
		req := request{
			ReqType:         "reload",
			Key:             "",
			Value:           "",
			PresentRootHash: "8ae58a88f5d3fdc46e4ee09f7e8c75b20e0439c97ccb0f62685173e50f8f3892",
			DeletedRootHash: "fd1f077f82a099c9680dbb31cabd3636",
		}

		rspBytes, err := sendReq(req)
		if err != nil {
			panic(err)
		}
		log.Println("content:", string(rspBytes))
	} else {
		panic("only start/reload is allowed")
	}

}

func initialData() {
	//try to put data
	req := request{ReqType: "put", Key: "dba", Value: "proxy"}
	rspBytes, err := sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to put data
	req = request{ReqType: "put", Key: "dbb", Value: "proxy"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to put data
	req = request{ReqType: "put", Key: "dbc", Value: "proxy"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to put data
	req = request{ReqType: "put", Key: "dbd", Value: "proxy"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to put data
	req = request{ReqType: "put", Key: "db", Value: "proxy"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))
}

func modifyData() {
	//try to get data
	req := request{ReqType: "get", Key: "db"}
	rspBytes, err := sendReq(req)
	if err != nil {
		panic(err)
	}
	log.Println("content:", string(rspBytes))

	fmt.Println("try to insert data")
	//try to insert data failed
	req = request{ReqType: "insert", Key: "db", Value: "proxy1"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to delete data
	fmt.Println("try to delete data")
	req = request{ReqType: "delete", Key: "db"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}
	log.Println("content:", string(rspBytes))

	//try to get the deleted data
	req = request{ReqType: "get", Key: "db"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}
	log.Println("content:", string(rspBytes))

	//try to insert data ,success
	fmt.Println("insert again start")
	req = request{ReqType: "insert", Key: "db", Value: "proxy1"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}

	log.Println("content:", string(rspBytes))
	fmt.Println("insert again finished")

	//try to get the inserted data
	req = request{ReqType: "get", Key: "db"}
	rspBytes, err = sendReq(req)
	if err != nil {
		panic(err)
	}
	log.Println("content:", string(rspBytes))

}

func persistData() {
	//try to persist data and
	req := request{ReqType: "save"}
	rspBytes, err := sendReq(req)
	if err != nil {
		panic(err)
	}
	log.Println("content:", string(rspBytes))
}

func sendReq(req request) ([]byte, error) {
	url := "http://127.0.0.1:8080/db-proxy"
	contentType := "application/json;charset=utf-8"

	b, err := json.Marshal(req)
	if err != nil {
		log.Println("json format error:", err)
		return nil, err
	}

	body := bytes.NewBuffer(b)
	resp, err := http.Post(url, contentType, body)
	if err != nil {
		log.Println("Post failed:", err)
		return nil, err
	}

	defer resp.Body.Close()
	content, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		log.Println("Read failed:", err)
		return nil, err
	}
	return content, nil
}
