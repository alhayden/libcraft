public class Test {
    public static void main(String... args) throws InterruptedException {
        System.out.println("Hello!");
        for(int i = 0; i < 10; i++) {
            Thread.sleep(1000);
            System.out.println("stayin' alive");
        }
        System.out.println("ded");
    }
}
