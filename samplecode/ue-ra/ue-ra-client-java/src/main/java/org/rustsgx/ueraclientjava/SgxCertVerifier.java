package org.rustsgx.ueraclientjava;

import java.util.List;
import javax.net.ssl.TrustManager;
import javax.net.ssl.X509TrustManager;
import java.security.cert.CertificateException;
import java.security.cert.X509Certificate;

public class SgxCertVerifier {
    TrustManager[] trustAllCerts;

    public SgxCertVerifier() {
        this.trustAllCerts = new TrustManager[] {
                new X509TrustManager() {
                    public X509Certificate[] getAcceptedIssuers() {
                        return new X509Certificate[0];
                    }
                    public void checkClientTrusted(X509Certificate[] certs, String authType) {}
                    public void checkServerTrusted(X509Certificate[] certs, String authType) throws CertificateException{
                        List<Byte> byteArray;
                        byteArray = CommonUtils.convertByte();
                        ServerCertData certData = VerifyMraCert.unmarshalByte(byteArray);

                        byte[] attnReportRaw = VerifyMraCert.verifyCert(certData.payload);
                        VerifyMraCert.verifyAtteReport(attnReportRaw,certData.pub_k);
                    }
                }
        };
    }
}
