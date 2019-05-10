package org.rustsgx.ueraclientjava;

import org.springframework.boot.autoconfigure.SpringBootApplication;

import javax.net.ssl.*;
import java.io.*;
import java.net.Socket;
import java.security.*;
import java.security.cert.Certificate;
import java.security.cert.X509Certificate;
import java.util.List;

@SpringBootApplication
public class UeRaClientJavaApplication {

	public static void main(String[] args) {

		HostnameVerifier hv = new HostnameVerifier() {
			public boolean verify(String hostname, SSLSession session) { return true; }
		};
		try{
			File crtFile = new File("./../cert/client.crt");
			List<X509Certificate> certificateChain = PemReader.readCertificateChain(crtFile);

			PrivateKey key = PemReader.getPemPrivateKey("./../cert/client.pkcs8","EC");

			KeyStore keyStore = KeyStore.getInstance("JKS");
			keyStore.load(null, null);
			keyStore.setKeyEntry("key", key, "".toCharArray(), certificateChain.stream().toArray(Certificate[]::new));

			KeyManagerFactory keyManagerFactory = KeyManagerFactory.getInstance("SunX509");
			keyManagerFactory.init(keyStore, "".toCharArray());

			SSLContext sc = SSLContext.getInstance("SSL");
			SgxCertVerifier sgxCertVerifier = new SgxCertVerifier();
			sc.init(keyManagerFactory.getKeyManagers(), sgxCertVerifier.trustAllCerts, new SecureRandom());

			SSLSocketFactory sf = sc.getSocketFactory();

			Socket s = sf.createSocket("127.0.0.1", 3443);

			// 向客户端回复信息
			DataOutputStream out = new DataOutputStream(s.getOutputStream());
			System.out.print("请输入:\t");
			// 发送键盘输入的一行
			String str = new BufferedReader(new InputStreamReader(System.in)).readLine();
			out.writeUTF(str);

			BufferedReader in = new BufferedReader(new InputStreamReader(s.getInputStream()));
			String x = in.readLine();
			System.out.println(x);

			out.close();
			in.close();
		}catch (Exception e){
			System.out.println(e.toString());
			return;
		}
		System.out.println("loadKeyStore success");
	}

}
