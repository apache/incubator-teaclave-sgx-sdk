package org.rustsgx.ueraclientjava;

import org.springframework.boot.autoconfigure.SpringBootApplication;

import javax.net.ssl.*;
import java.io.*;
import java.net.Socket;
import java.security.*;

@SpringBootApplication
public class UeRaClientJavaApplication {

    public static void main(String[] args) {
        System.out.println("Starting ue-ra-client-java");

        try {
            SSLContext sc = SSLContext.getInstance("SSL");
            SgxCertVerifier sgxCertVerifier = new SgxCertVerifier();
            sc.init(sgxCertVerifier.keyManagerFactory.getKeyManagers(), sgxCertVerifier.trustAllCerts, new SecureRandom());

            SSLSocketFactory sf = sc.getSocketFactory();

            System.out.println("Connecting to  localhost:3443");
            Socket s = sf.createSocket("127.0.0.1", 3443);

            DataOutputStream out = new DataOutputStream(s.getOutputStream());
            String str = "hello ue-ra-java-client";
            out.write(str.getBytes());

            BufferedReader in = new BufferedReader(new InputStreamReader(s.getInputStream()));
            String x = in.readLine();
            System.out.printf("server replied:  %s\n", x);

            out.close();
            in.close();
        } catch (Exception e) {
            System.out.println(e.toString());
            System.exit(0);
        }
    }

}
