/** 判断当前是否运行在移动端 WebView（Android / iOS） */
export function isMobilePlatform(): boolean {
  return (
    typeof navigator !== "undefined" &&
    /android|iphone|ipad|ipod/i.test(navigator.userAgent)
  );
}
