package org.rustsgx.mio;

import org.apache.commons.io.IOUtils;
import org.apache.http.NameValuePair;
import org.apache.http.client.utils.URIBuilder;
import org.apache.http.message.BasicNameValuePair;

import javax.net.ssl.HttpsURLConnection;
import java.io.IOException;
import java.io.StringWriter;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;

public class AppClient {

    private String scheme = null;
    private String url = null;
    private int port = 80;


    public AppClient(final String scheme, final String url, final int port) {
        this.scheme = scheme;
        this.url = url;
        this.port = port;
    }

    public String request(final String requestType,
                          final String endpoint,
                          final HashMap<String, String> parameters) throws URISyntaxException, IOException {
        HttpURLConnection conn = null;

        try {

            List<NameValuePair> nameValues = new ArrayList<NameValuePair>();
            for (String identifier : parameters.keySet()) {
                NameValuePair pair = new BasicNameValuePair(identifier, parameters.get(identifier));
                nameValues.add(pair);
            }

            URI uri = new URIBuilder()
                    .setScheme(this.scheme)
                    .setHost(this.url)
                    .setPort(this.port)
                    .setPath(endpoint)
                    .setParameters(nameValues)
                    .build();


            if (requestType.equals("https")) {
                HttpsURLConnection httpsConn = (HttpsURLConnection) uri.toURL().openConnection();
                httpsConn.setRequestMethod(requestType);
                httpsConn.setDoInput(true);
                httpsConn.connect();
                conn = httpsConn;
            } else {
                HttpURLConnection httpConn = (HttpURLConnection) uri.toURL().openConnection();
                httpConn.setRequestMethod(requestType);
                httpConn.setDoInput(true);
                httpConn.connect();
                conn = httpConn;
            }

            StringWriter writer = new StringWriter();
            IOUtils.copy(conn.getInputStream(), writer, "UTF-8");

            return writer.toString();

        } finally {
            if (conn != null) {
                conn.disconnect();
            }
        }
    }
}
