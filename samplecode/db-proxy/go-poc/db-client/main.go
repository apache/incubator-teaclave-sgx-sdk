package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
)

type request struct {
	ReqType string `json:"req_type"`
	Key     string `json:"key"`
	Value   string `json:"value"`
}

type response struct {
	RspStatus bool `json:rsp_status`
}

func main() {

	fmt.Println("start db-client")

	//try to put data
	rep := request{ReqType: "put", Key: "db", Value: "proxy"}
	rspBytes, err := sendReq(rep)
	if err!= nil{
		panic(err)
	}

	log.Println("content:", string(rspBytes))

	//try to get data
	rep = request{ReqType: "get", Key: "db"}
	rspBytes, err = sendReq(rep)
	if err!= nil{
		panic(err)
	}
	log.Println("content:", string(rspBytes))

	//try to delete data
	rep = request{ReqType: "delete", Key: "db"}
	rspBytes, err = sendReq(rep)
	if err!= nil{
		panic(err)
	}
	log.Println("content:", string(rspBytes))

	//try to get the deleted data
	rep = request{ReqType: "get", Key: "db"}
	rspBytes, err = sendReq(rep)
	if err!= nil{
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
