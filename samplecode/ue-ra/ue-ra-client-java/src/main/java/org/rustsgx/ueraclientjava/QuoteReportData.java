package org.rustsgx.ueraclientjava;

//TODO: add more origin filed if needed
public class QuoteReportData {
    private int version;
    private int signType;
    private QuoteReportBody quoteReportBody;

    public void setVersion(int version) {
        this.version = version;
    }

    public void setSignType(int signType) {
        this.signType = signType;
    }

    public void setQuoteReportBody(QuoteReportBody quoteReportBody) {
        this.quoteReportBody = quoteReportBody;
    }

    public int getVersion() {
        return version;
    }

    public int getSignType() {
        return signType;
    }

    public QuoteReportBody getQuoteReportBody() {
        return quoteReportBody;
    }

    //TODO: add more origin filed if needed
    class QuoteReportBody {
        private String mrEnclave;
        private String mrSigner;
        private String reportData;

        public void setMrEnclave(String mrEnclave) {
            this.mrEnclave = mrEnclave;
        }

        public void setMrSigner(String mrSigner) {
            this.mrSigner = mrSigner;
        }

        public void setReportData(String reportData) {
            this.reportData = reportData;
        }

        public String getMrEnclave() {
            return mrEnclave;
        }

        public String getMrSigner() {
            return mrSigner;
        }

        public String getReportData() {
            return reportData;
        }
    }

    public void pareReport(byte[] quoteRep, String repHex, QuoteReportData quoteReportData) {
        quoteReportData.quoteReportBody = new QuoteReportBody();
        quoteReportData.version = Byte.toUnsignedInt(quoteRep[0]);
        quoteReportData.signType = Byte.toUnsignedInt(quoteRep[1]);
        quoteReportData.quoteReportBody.mrEnclave = repHex.substring(224, 288);
        quoteReportData.quoteReportBody.mrSigner = repHex.substring(352, 416);
        quoteReportData.quoteReportBody.reportData = repHex.substring(736, 864);
    }
}
