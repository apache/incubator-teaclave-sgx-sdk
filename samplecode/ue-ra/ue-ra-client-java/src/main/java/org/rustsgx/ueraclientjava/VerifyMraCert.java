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
import java.util.Base64;
import java.util.List;

public class VerifyMraCert {
    public static ServerCertData unmarshalByte(List<Byte> certList){
        // Search for Public Key prime256v1 OID
        String[] prime256v1_oid_string = new String[]{"0x06",
                "0x08", "0x2a", "0x86", "0x48", "0xce", "0x3d", "0x03", "0x01", "0x07"};

        List<Byte> prime256v1_oid = CommonUtils.string2BytesList(prime256v1_oid_string);

        int offset = CommonUtils.getIndexOf(certList,prime256v1_oid);
        offset += 11; // 10 + TAG (0x03)

        // Obtain Public Key length
        int length = Byte.toUnsignedInt(certList.get(offset));
        if (length > Byte.toUnsignedInt(CommonUtils.hexToByte("80"))){
            length = Byte.toUnsignedInt(certList.get(offset+1))* 256 +
                    Byte.toUnsignedInt(certList.get(offset+2));
            offset += 2;
        }

        // Obtain Public Key
        offset += 1;
        byte[] pub_k = CommonUtils.list2array(certList.subList(offset+2,offset+length)); // skip "00 04"

        String[] ns_cmt_oid_string = new String[]{"0x06",
                "0x09", "0x60", "0x86", "0x48", "0x01", "0x86", "0xf8", "0x42", "0x01", "0x0d"};
        List<Byte> ns_cmt_oid = CommonUtils.string2BytesList(ns_cmt_oid_string);
        offset = CommonUtils.getIndexOf(certList,ns_cmt_oid);
        offset += 12; // 10 + TAG (0x03)


        // Obtain Netscape Comment length
        length = Byte.toUnsignedInt(certList.get(offset));
        if (length > Byte.toUnsignedInt(CommonUtils.hexToByte("80"))) {
            length = Byte.toUnsignedInt(certList.get(offset+1))*256+
                    Byte.toUnsignedInt(certList.get(offset+2));
            offset += 2;
        }

        offset += 1;
        List<Byte> payload = certList.subList(offset,offset+length);

        return new ServerCertData(payload,pub_k);
    }

    public static byte[] verifyCert(List<Byte> payload){
        Base64.Decoder decoder = Base64.getDecoder();

        int startIndex = payload.indexOf(CommonUtils.hexToByte("7c"));
        int endIndex = payload.lastIndexOf(CommonUtils.hexToByte("7c"));
        byte[] attnReportRaw = CommonUtils.list2array(payload.subList(0,startIndex));
        byte[] sigRaw = CommonUtils.list2array(payload.subList(startIndex+1,endIndex));
        byte[] sig = decoder.decode(sigRaw);
        System.out.println("sig:");
        for(int i=0;i<sig.length;i++){
            System.out.printf("%02x",sig[i]);
        }
        System.out.println("");
        System.out.println("sigCert");
        byte[] sigCertRaw = CommonUtils.list2array(payload.subList(endIndex+1,payload.size()));
        byte[] sigCert = decoder.decode(sigCertRaw);
        for(int i=0;i<sigCert.length;i++){
            System.out.printf("%02x",sigCert[i]);
        }
        System.out.println("");

        X509Certificate server, provider;

        try{
            CertificateFactory cf = CertificateFactory.getInstance("X509");
            server = (X509Certificate) cf.generateCertificate(new ByteArrayInputStream(sigCert));
            System.out.println(server.getSigAlgName());

            FileReader reader = new FileReader("./../cert/AttestationReportSigningCACert.pem");
            PEMParser pemParser = new PEMParser(reader);
            X509CertificateHolder x509CertificateHolder = (X509CertificateHolder) pemParser.readObject();
            provider = new JcaX509CertificateConverter().getCertificate( x509CertificateHolder );

        }catch(Exception e){
            System.out.println(e);
            return null;
        }

        try{
            server.verify(provider.getPublicKey());
        }catch (Exception e){
            System.out.println(e);
            return null;
        }

        System.out.println("cert is Good");

        try{
            Signature signature = Signature.getInstance(server.getSigAlgName());
            signature.initVerify(server);
            signature.update(attnReportRaw);
            boolean ok = signature.verify(sig);
            if(ok == false){
                return null;
            }
        }catch(Exception e){
            return null;
        }
        System.out.println("signature is Good");


        return attnReportRaw;
    }

    public static void verifyAtteReport(byte[] attnReportRaw,byte[] pubK){
        Gson gson = new Gson();

        String attReportJson = new String();
        for (int i=0;i<attnReportRaw.length;i++){
            attReportJson += (char)attnReportRaw[i];
        }
        System.out.println(attReportJson);
        SgxQuoteReport sgxQr;
        try{
            sgxQr = gson.fromJson(attReportJson,SgxQuoteReport.class);
        }catch (Exception e){
            System.out.println(e);
            return;
        }


        //1 Check timestamp is within 24H
        String timeFixed = sgxQr.getTimestamp()+"Z";
        DateTime dateTime = new DateTime(timeFixed);
        DateTime now = new DateTime();

        Interval interval = new Interval(dateTime.getMillis(), now.getMillis());
        System.out.println(Seconds.secondsIn(interval).getSeconds());

        //2 Verify quote status (mandatory field)
        byte[] pfBlob = HexBin.decode(sgxQr.getPlatformInfoBlob());
        for(int i=0;i<pfBlob.length;i++){
            System.out.printf("%02x",pfBlob[i]);
        }
        System.out.println("");

        // 3 Verify quote body
        Base64.Decoder decoder = Base64.getDecoder();
        byte []qb = decoder.decode(sgxQr.getIsvEnclaveQuoteBody());

        String qbString = new String();
        String qbBytes = new String();
        String pubKeyString = new String();
        for(int i=0;i<qb.length;i++){
            qbBytes += String.format("%d, ",Byte.toUnsignedInt(qb[i]));
            qbString += String.format("%02x",qb[i]);
        }
        for(int i=0;i<pubK.length;i++){
            pubKeyString += String.format("%02x",pubK[i]);
        }

        System.out.println("Quote = ["+ qbBytes.substring(0,qbBytes.length()-2) + "]");
        System.out.println(qbString);
        System.out.println(pubKeyString);
        System.out.println(sgxQr.getIsvEnclaveQuoteBody());
    }
}
