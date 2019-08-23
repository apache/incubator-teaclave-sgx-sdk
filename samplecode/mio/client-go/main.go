package main

import (
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"io/ioutil"
	"net/http"
	"sync"
)

func main() {
	pool := x509.NewCertPool()
	caCertPath := "./ca.cert"

	caCrt, err := ioutil.ReadFile(caCertPath)
	if err != nil {
		fmt.Println("ReadFile err:", err)
		return
	}
	pool.AppendCertsFromPEM(caCrt)

	tr := &http.Transport{
		TLSClientConfig: &tls.Config{RootCAs: pool},
	}

	wg := sync.WaitGroup{}
	wg.Add(20)

	for i:=0;i<20;i++{
		go func() {
			client := &http.Client{Transport: tr}
			resp, err := client.Get("https://localhost:8443")
			if err != nil {
				fmt.Println("Get error:", err)
				return
			}
			defer resp.Body.Close()
			body, err := ioutil.ReadAll(resp.Body)
			fmt.Println(string(body))
			wg.Done()
		}()
	}

	wg.Wait()
}
