package org.rustsgx.ueraclientjava;

public class SgxQuoteReport {
    private String id;
    private String timestamp;
    private int version;
    private String isvEnclaveQuoteStatus;
    private String platformInfoBlob;
    private String isvEnclaveQuoteBody;

    public String getId() {
        return id;
    }

    public String getTimestamp() {
        return timestamp;
    }

    public int getVersion() {
        return version;
    }

    public String getIsvEnclaveQuoteStatus() {
        return isvEnclaveQuoteStatus;
    }

    public String getPlatformInfoBlob() {
        return platformInfoBlob;
    }

    public String getIsvEnclaveQuoteBody() {
        return isvEnclaveQuoteBody;
    }
}