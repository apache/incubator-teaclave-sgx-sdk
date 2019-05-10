package org.rustsgx.ueraclientjava;

import java.io.*;
import java.nio.CharBuffer;
import java.security.*;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.GeneralSecurityException;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static java.nio.charset.StandardCharsets.US_ASCII;
import static java.util.regex.Pattern.CASE_INSENSITIVE;

public class PemReader {

    private static final Pattern CERT_PATTERN = Pattern.compile(
            "-+BEGIN\\s+.*CERTIFICATE[^-]*-+(?:\\s|\\r|\\n)+" + // Header
                    "([a-z0-9+/=\\r\\n]+)" +                    // Base64 text
                    "-+END\\s+.*CERTIFICATE[^-]*-+",            // Footer
            CASE_INSENSITIVE);

    public static List<X509Certificate> readCertificateChain(File certificateChainFile)
            throws IOException, GeneralSecurityException {
        String contents = readFile(certificateChainFile);

        Matcher matcher = CERT_PATTERN.matcher(contents);
        CertificateFactory certificateFactory = CertificateFactory.getInstance("X.509");
        List<X509Certificate> certificates = new ArrayList<>();

        int start = 0;
        while (matcher.find(start)) {
            byte[] buffer = base64Decode(matcher.group(1));
            certificates.add((X509Certificate) certificateFactory.generateCertificate(new ByteArrayInputStream(buffer)));
            start = matcher.end();
        }

        return certificates;
    }

    private static String readFile(File file)
            throws IOException {
        try (Reader reader = new InputStreamReader(new FileInputStream(file), US_ASCII)) {
            StringBuilder stringBuilder = new StringBuilder();

            CharBuffer buffer = CharBuffer.allocate(2048);
            while (reader.read(buffer) != -1) {
                buffer.flip();
                stringBuilder.append(buffer);
                buffer.clear();
            }
            return stringBuilder.toString();
        }
    }

    private static byte[] base64Decode(String base64) {
        return Base64.getMimeDecoder().decode(base64.getBytes(US_ASCII));
    }

    public static PrivateKey getPemPrivateKey(String filename, String algorithm) throws Exception {
        File f = new File(filename);
        FileInputStream fis = new FileInputStream(f);
        DataInputStream dis = new DataInputStream(fis);
        byte[] keyBytes = new byte[(int) f.length()];
        dis.readFully(keyBytes);
        dis.close();

        String temp = new String(keyBytes);
        String privKeyPEM = temp.replace("-----BEGIN PRIVATE KEY-----\n", "");
        privKeyPEM = privKeyPEM.replace("-----END PRIVATE KEY-----", "");
        //System.out.println("Private key\n"+privKeyPEM);

        org.bouncycastle.util.encoders.Base64 b64 = new org.bouncycastle.util.encoders.Base64();
        byte[] decoded = b64.decode(privKeyPEM);

        PKCS8EncodedKeySpec spec = new PKCS8EncodedKeySpec(decoded);
        KeyFactory kf = KeyFactory.getInstance(algorithm);
        return kf.generatePrivate(spec);
    }
}