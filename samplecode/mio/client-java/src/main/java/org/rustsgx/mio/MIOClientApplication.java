package org.rustsgx.mio;
import org.springframework.boot.autoconfigure.SpringBootApplication;

import java.net.Socket;
import java.util.HashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;


@SpringBootApplication
public class MIOClientApplication {
    public static void main(String[] args) {
        System.out.println("This is a mio client application");

        SSLHelper.loadCerts("ca.cert");

        int count = 20;
        ExecutorService service = Executors.newFixedThreadPool(count);
        for (int i = 0; i < count; i++) {
            service.execute(() -> {
                try {
                    AppClient appClient = new AppClient("https", "localhost", 8443);
                    System.out.println(appClient.request("GET", "/", new HashMap<>()));
                    Thread.sleep(10000);
                } catch (Exception ex) {
                    ex.printStackTrace();
                }
            });
        }
        service.shutdown();
        service.awaitTermination(10000000, TimeUnit.SECONDS);
    }
}
