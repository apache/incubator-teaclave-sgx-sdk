package org.rustsgx.ueraclientjava;

import java.io.File;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.Certificate;
import java.util.ArrayList;
import java.util.List;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.TrustManager;
import javax.net.ssl.X509TrustManager;
import java.security.cert.CertificateException;
import java.security.cert.X509Certificate;

public class SgxCertVerifier {
    TrustManager[] trustAllCerts;
    KeyManagerFactory keyManagerFactory;

    public SgxCertVerifier() throws Exception {
        //init keyManagerFactory
        try {
            File crtFile = new File("./../cert/client.crt");
            List<X509Certificate> certificateChain = PemReader.readCertificateChain(crtFile);
            PrivateKey key = PemReader.getPemPrivateKey("./../cert/client.pkcs8", "EC");

            KeyStore keyStore = KeyStore.getInstance("JKS");
            keyStore.load(null, null);
            keyStore.setKeyEntry("key", key, "".toCharArray(), certificateChain.stream().toArray(Certificate[]::new));

            this.keyManagerFactory = KeyManagerFactory.getInstance("SunX509");
            this.keyManagerFactory.init(keyStore, "".toCharArray());
        } catch (Exception e) {
            System.out.print(e.toString());
            throw e;
        }

        //init TrustManager
        this.trustAllCerts = new TrustManager[]{
                new X509TrustManager() {
                    public X509Certificate[] getAcceptedIssuers() {
                        return new X509Certificate[0];
                    }

                    public void checkClientTrusted(X509Certificate[] certs, String authType) {
                    }

                    public void checkServerTrusted(X509Certificate[] certs, String authType) throws CertificateException {
                        CommonUtils.printCert(certs[0].getEncoded());
                        List<Byte> byteArray = new ArrayList<Byte>();
                        for (int i = 0; i < certs[0].getEncoded().length; i++) {
                            byteArray.add(certs[0].getEncoded()[i]);
                        }
                        // get the pubkey and payload from raw data
                        ServerCertData certData = VerifyMraCert.unmarshalByte(byteArray);

                        try {
                            // Load Intel CA, Verify Cert and Signature
                            byte[] attnReportRaw = VerifyMraCert.verifyCert(certData.payload);

                            // Verify attestation report
                            VerifyMraCert.verifyAtteReport(attnReportRaw, certData.pub_k);
                        } catch (Exception e) {
                            System.out.println(e.toString());
                            System.exit(0);
                        }
                    }
                }
        };
    }
}
