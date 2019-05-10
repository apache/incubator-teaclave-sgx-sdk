package main

import (
	"fmt"
	"strconv"
)

type QuoteReport struct {
	ID                    string `json:"id"`
	Timestamp             string `json:"timestamp"`
	Version               int    `json:"version"`
	IsvEnclaveQuoteStatus string `json:"isvEnclaveQuoteStatus"`
	PlatformInfoBlob      string `json:"platformInfoBlob"`
	IsvEnclaveQuoteBody   string `json:"isvEnclaveQuoteBody"`
}

//TODO: add more origin field if needed
type QuoteReportData struct {
	version    int
	signType   int
	reportBody QuoteReportBody
}

//TODO: add more origin filed if needed
type QuoteReportBody struct {
	mrEnclave  string
	mrSigner   string
	reportData string
}

type PlatformInfoBlob struct {
	Sgx_epid_group_flags       uint8             `json:"sgx_epid_group_flags"`
	Sgx_tcb_evaluation_flags   uint32            `json:"sgx_tcb_evaluation_flags"`
	Pse_evaluation_flags       uint32            `json:"pse_evaluation_flags"`
	Latest_equivalent_tcb_psvn string            `json:"latest_equivalent_tcb_psvn"`
	Latest_pse_isvsvn          string            `json:"latest_pse_isvsvn"`
	Latest_psda_svn            string            `json:"latest_psda_svn"`
	Xeid                       uint32            `json:"xeid"`
	Gid                        uint32            `json:"gid"`
	Sgx_ec256_signature_t      SGXEC256Signature `json:"sgx_ec256_signature_t"`
}

type SGXEC256Signature struct {
	Gx string `json:"gx"`
	Gy string `json:"gy"`
}

// directly read from []byte
func parseReport(quoteBytes []byte, quoteHex string) *QuoteReportData {
	qrData := &QuoteReportData{reportBody: QuoteReportBody{}}
	qrData.version = int(quoteBytes[0])
	qrData.signType = int(quoteBytes[2])
	qrData.reportBody.mrEnclave = quoteHex[224:288]
	qrData.reportBody.mrSigner = quoteHex[352:416]
	qrData.reportBody.reportData = quoteHex[736:864]
	return qrData
}

// directly read from []byte
func parsePlatform(piBlobByte []byte) *PlatformInfoBlob {
	piBlob := &PlatformInfoBlob{Sgx_ec256_signature_t: SGXEC256Signature{}}
	piBlob.Sgx_epid_group_flags = uint8(piBlobByte[0])
	piBlob.Sgx_tcb_evaluation_flags = computeDec(piBlobByte[1:3])
	piBlob.Pse_evaluation_flags = computeDec(piBlobByte[3:5])
	piBlob.Latest_equivalent_tcb_psvn = bytesToString(piBlobByte[5:23])
	piBlob.Latest_pse_isvsvn = bytesToString(piBlobByte[23:25])
	piBlob.Latest_psda_svn = bytesToString(piBlobByte[25:29])
	piBlob.Xeid = computeDec(piBlobByte[29:33])
	piBlob.Gid = computeDec(piBlobByte[33:37])
	piBlob.Sgx_ec256_signature_t.Gx = bytesToString(piBlobByte[37:69])
	piBlob.Sgx_ec256_signature_t.Gy = bytesToString(piBlobByte[69:])

	return piBlob
}

func computeDec(piBlobSlice []byte) uint32 {
	var hexString string
	for i := len(piBlobSlice)-1; i >= 0; i-- {
		hexString += fmt.Sprintf("%02x", piBlobSlice[i])
	}
	s, _ := strconv.ParseInt(hexString, 16, 32)

	return uint32(s)
}

func bytesToString(byteSlice []byte) string {
	var byteString string
	for i := 0; i < len(byteSlice); i++ {
		byteString += strconv.Itoa(int(byteSlice[i])) + ", "
	}
	byteString = "[" + byteString[:len(byteString)-2] + "]"
	return byteString
}
