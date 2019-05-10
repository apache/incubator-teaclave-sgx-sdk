package org.rustsgx.ueraclientjava;

import com.google.gson.Gson;
import com.sun.org.apache.xerces.internal.impl.dv.util.HexBin;
import org.bouncycastle.cert.X509CertificateHolder;
import org.bouncycastle.cert.jcajce.JcaX509CertificateConverter;
import org.bouncycastle.openssl.PEMParser;
import org.joda.time.DateTime;
import org.joda.time.Interval;
import org.joda.time.Seconds;

import java.io.ByteArrayInputStream;
import java.io.FileReader;
import java.security.Signature;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.util.Arrays;
import java.util.Base64;
import java.util.List;

public class VerifyMraCert {
    public static ServerCertData unmarshalByte(List<Byte> certList) {
        // Search for Public Key prime256v1 OID
        String[] prime256v1_oid_string = new String[]{"0x06",
                "0x08", "0x2a", "0x86", "0x48", "0xce", "0x3d", "0x03", "0x01", "0x07"};

        List<Byte> prime256v1_oid = CommonUtils.string2BytesList(prime256v1_oid_string);
        int offset = CommonUtils.getIndexOf(certList, prime256v1_oid);
        offset += 11; // 10 + TAG (0x03)

        // Obtain Public Key length
        int length = Byte.toUnsignedInt(certList.get(offset));
        if (length > Byte.toUnsignedInt(CommonUtils.hexToByte("80"))) {
            length = Byte.toUnsignedInt(certList.get(offset + 1)) * 256 +
                    Byte.toUnsignedInt(certList.get(offset + 2));
            offset += 2;
        }

        // Obtain Public Key
        offset += 1;
        byte[] pub_k = CommonUtils.list2array(certList.subList(offset + 2, offset + length)); // skip "00 04"

        String[] ns_cmt_oid_string = new String[]{"0x06",
                "0x09", "0x60", "0x86", "0x48", "0x01", "0x86", "0xf8", "0x42", "0x01", "0x0d"};
        List<Byte> ns_cmt_oid = CommonUtils.string2BytesList(ns_cmt_oid_string);
        offset = CommonUtils.getIndexOf(certList, ns_cmt_oid);
        offset += 12; // 10 + TAG (0x03)


        // Obtain Netscape Comment length
        length = Byte.toUnsignedInt(certList.get(offset));
        if (length > Byte.toUnsignedInt(CommonUtils.hexToByte("80"))) {
            length = Byte.toUnsignedInt(certList.get(offset + 1)) * 256 +
                    Byte.toUnsignedInt(certList.get(offset + 2));
            offset += 2;
        }

        offset += 1;
        List<Byte> payload = certList.subList(offset, offset + length);

        return new ServerCertData(payload, pub_k);
    }

    public static byte[] verifyCert(List<Byte> payload) throws Exception {
        Base64.Decoder decoder = Base64.getDecoder();

        int startIndex = payload.indexOf(CommonUtils.hexToByte("7c"));
        int endIndex = payload.lastIndexOf(CommonUtils.hexToByte("7c"));
        byte[] attnReportRaw = CommonUtils.list2array(payload.subList(0, startIndex));
        byte[] sigRaw = CommonUtils.list2array(payload.subList(startIndex + 1, endIndex));
        byte[] sig = decoder.decode(sigRaw);
        byte[] sigCertRaw = CommonUtils.list2array(payload.subList(endIndex + 1, payload.size()));
        byte[] sigCert = decoder.decode(sigCertRaw);
        X509Certificate server, provider;

        try {
            CertificateFactory cf = CertificateFactory.getInstance("X509");
            server = (X509Certificate) cf.generateCertificate(new ByteArrayInputStream(sigCert));
            FileReader reader = new FileReader("./../cert/AttestationReportSigningCACert.pem");
            PEMParser pemParser = new PEMParser(reader);
            X509CertificateHolder x509CertificateHolder = (X509CertificateHolder) pemParser.readObject();
            provider = new JcaX509CertificateConverter().getCertificate(x509CertificateHolder);
            server.verify(provider.getPublicKey());
        } catch (Exception e) {
            throw e;
        }

        System.out.println("Cert is good");
        try {
            Signature signature = Signature.getInstance(server.getSigAlgName());
            signature.initVerify(server);
            signature.update(attnReportRaw);
            boolean ok = signature.verify(sig);
            if (ok == false) {
                throw new Exception("failed to parse root certificate");
            }
        } catch (Exception e) {
            throw e;
        }
        System.out.println("Signature good");

        return attnReportRaw;
    }

    public static void verifyAtteReport(byte[] attnReportRaw, byte[] pubK) throws Exception {
        //extract data from attReportJson
        Gson gson = new Gson();
        String attReportJson = new String();
        for (int i = 0; i < attnReportRaw.length; i++) {
            attReportJson += (char) attnReportRaw[i];
        }
        SgxQuoteReport sgxQr;
        try {
            sgxQr = gson.fromJson(attReportJson, SgxQuoteReport.class);
        } catch (Exception e) {
            throw e;
        }

        //1 Check timestamp is within 24H
        if (sgxQr.getTimestamp().length() != 0) {
            String timeFixed = sgxQr.getTimestamp() + "Z";
            DateTime dateTime = new DateTime(timeFixed);
            DateTime now = new DateTime();
            Interval interval = new Interval(dateTime.getMillis(), now.getMillis());
            System.out.printf("Time diff =  %d\n", Seconds.secondsIn(interval).getSeconds());
        } else {
            throw new Exception("Failed to fetch timestamp from attestation report");
        }


        //2 Verify quote status (mandatory field)
        if (sgxQr.getIsvEnclaveQuoteStatus().length() != 0) {
            System.out.printf("isvEnclaveQuoteStatus = %s\n", sgxQr.getIsvEnclaveQuoteStatus());
            switch (sgxQr.getIsvEnclaveQuoteStatus()) {
                case "OK":
                    break;
                case "GROUP_OUT_OF_DATE":
                case "GROUP_REVOKED":
                case "CONFIGURATION_NEEDED":
                    if (sgxQr.getPlatformInfoBlob().length() != 0) {
                        byte[] pfBlob = HexBin.decode(sgxQr.getPlatformInfoBlob());
                        PlatformInfoBlob platformInfoBlob = new PlatformInfoBlob();
                        platformInfoBlob.parsePlatInfo(Arrays.copyOfRange(pfBlob, 4, pfBlob.length), platformInfoBlob);
                        System.out.printf("Platform info is: %s\n", gson.toJson(platformInfoBlob));
                    } else {
                        throw new Exception("Failed to fetch platformInfoBlob from attestation report");
                    }
                    break;
                default:
                    throw new Exception("SGX_ERROR_UNEXPECTED");
            }
        } else {
            throw new Exception("Failed to fetch isvEnclaveQuoteStatus from attestation report");
        }


        // 3 Verify quote body
        if (sgxQr.getIsvEnclaveQuoteBody().length() != 0) {
            Base64.Decoder decoder = Base64.getDecoder();
            byte[] qb = decoder.decode(sgxQr.getIsvEnclaveQuoteBody());
            String qbString = new String();
            String qbBytes = new String();
            String pubKeyString = new String();
            for (int i = 0; i < qb.length; i++) {
                qbBytes += String.format("%d, ", Byte.toUnsignedInt(qb[i]));
                qbString += String.format("%02x", qb[i]);
            }
            for (int i = 0; i < pubK.length; i++) {
                pubKeyString += String.format("%02x", pubK[i]);
            }

            QuoteReportData quoteReportData = new QuoteReportData();
            quoteReportData.pareReport(qb, qbString, quoteReportData);
            System.out.println("Quote = [" + qbBytes.substring(0, qbBytes.length() - 2) + "]");
            System.out.printf("sgx quote version = %s\n", quoteReportData.getVersion());
            System.out.printf("sgx quote signature type = %s\n", quoteReportData.getSignType());
            System.out.printf("sgx quote report_data = %s\n", quoteReportData.getQuoteReportBody().getReportData());
            System.out.printf("sgx quote mr_enclave = %s\n", quoteReportData.getQuoteReportBody().getMrEnclave());
            System.out.printf("sgx quote mr_signer = %s\n", quoteReportData.getQuoteReportBody().getMrSigner());
            System.out.printf("Anticipated public key = %s\n", pubKeyString);

            if (pubKeyString.equals(quoteReportData.getQuoteReportBody().getReportData())) {
                System.out.println("ue RA done!");
            }
        } else {
            throw new Exception("Failed to fetch isvEnclaveQuoteBody from attestation report");
        }

    }
}
