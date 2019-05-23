package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/pkg/errors"
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
	ErrorInfo string `json:errorInfo`
}

//hmacPayload is used to compute hmac
type hmacPayload struct {
	Key            string `json:key`
	Value          string `json:Value`
	PresentCounter int64  `json:present_counter`
}

//storePayload is used to store value in the db
type storePayload struct {
	Value string `json:value`
	Tag   string `json:tag`
	Ctr   int64  `json:ctr`
}

func main() {
	hmac_key := []byte{
		0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
		0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
		0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
		0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
		0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
		0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
		0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
		0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
	}

	//will be replaced by MB-Tree
	present := make(map[string]int64)
	deleted := make(map[string]int64)

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
		rsp := validate(db, b, hmac_key, present, deleted)
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

func validate(db *leveldb.DB, reqByte, hmac_key []byte, present, deleted map[string]int64) response {
	rsp := response{RspStatus: true}
	var err error
	var data []byte

	req := &request{}
	err = json.Unmarshal(reqByte, req)

	switch req.ReqType {
	case "get":
		data, err = db.Get([]byte(req.Key), nil)
		sp := storePayload{}
		if err == nil {
			err := json.Unmarshal(data, &sp)
			if err == nil {
			}
		} else {
			break
		}
		//verify hmac
		hmacPayload := hmacPayload{Key: req.Key, Value: sp.Value, PresentCounter: sp.Ctr}
		hmacByte, _ := json.Marshal(hmacPayload)
		tagByte, _ := hex.DecodeString(sp.Tag)
		if ValidMAC(hmacByte, tagByte, hmac_key) && sp.Ctr == present[req.Key] {
			rsp.Data = sp.Value
		} else {
			fmt.Println("validate failed")
			err = errors.New("validate failed")
		}
		break
	case "put":
		//compute tag of k,v,counter
		hmacPayload := hmacPayload{Key: req.Key, Value: req.Value, PresentCounter: present[req.Key] + 1}
		hmacByte, _ := json.Marshal(hmacPayload)
		tagByte := computeHMAC(hmacByte, hmac_key)
		tagString := fmt.Sprintf("%02x", tagByte)

		//try to put it into kvdb
		storePayload := storePayload{Value: req.Value, Ctr: present[req.Key] + 1, Tag: tagString}
		spByte, _ := json.Marshal(storePayload)
		err = db.Put([]byte(req.Key), spByte, nil)

		//update present if there is no error
		if err == nil {
			present[req.Key] = present[req.Key] + 1
		}
		break
	case "delete":
		err = db.Delete([]byte(req.Key), nil)
		if err == nil {
			deleted[req.Key] = present[req.Key]
			delete(present, req.Key)
		}
		break
	case "insert":
		if _, ok := present[req.Key]; ok {
			err = errors.New("key existed in present when called insert")
		} else {
			var ctr int64
			if _, ok := deleted[req.Key]; ok {
				ctr = deleted[req.Key] + 1
			} else {
				ctr = 0
			}
			hmacPayload := hmacPayload{Key: req.Key, Value: req.Value, PresentCounter: ctr}
			hmacByte, _ := json.Marshal(hmacPayload)
			tagByte := computeHMAC(hmacByte, hmac_key)
			tagString := fmt.Sprintf("%02x", tagByte)

			//try to insert it into kvdb
			storePayload := storePayload{Value: req.Value, Ctr: ctr, Tag: tagString}
			spByte, _ := json.Marshal(storePayload)
			err = db.Put([]byte(req.Key), spByte, nil)
			if err == nil {
				present[req.Key] = ctr
				delete(deleted, req.Key)
			}
		}
		break
	default:
	}

	if err != nil {
		rsp.RspStatus = false
		rsp.ErrorInfo = "key_missing_error"
		fmt.Printf("request failed:%s\n", string(reqByte))
	} else {
		fmt.Printf("request successd:%s\n", string(reqByte))
	}
	return rsp
}

func ValidMAC(message, messageMAC, key []byte) bool {
	mac := hmac.New(sha256.New, key)
	mac.Write(message)
	expectedMAC := mac.Sum(nil)
	return hmac.Equal(messageMAC, expectedMAC)
}

func computeHMAC(message, key []byte) []byte {
	mac := hmac.New(sha256.New, key)
	mac.Write(message)
	expectedMAC := mac.Sum(nil)
	return expectedMAC
}
