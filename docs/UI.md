# UI

* 주요 UI window는 AppView와 PopupView 가 있다.


## AppView

### Work tab:
![alt text](docs/res/image-work.png)

### Report tab:
![alt text](docs/res/image-report.png)

left 사이드바에 work / report를 선택할 수 있도록.
 - work 선택 시 placeholder / report 선택 시 하루 단위의 report 목록 조회 가능.
right 사이드바는 event list이다. 특정한 예를 들어 - '메신저 접속함' / '사내 게시판 접속' / '스프레드시트 on' 등의 이벤트 발생 시 log를 띄운다.

# Styling Guide
레이아웃을 짤 때 margin이나 gap은 절대 쓰지 말고, <div> 대신 시맨틱 태그 위주로 사용한다.
요소 간의 구분은 오직 1px의 얇은 선(border, divide)과 내부 padding으로만 처리해서 선형적이고 에스테틱한 느낌을 강조한다.
대신 구분 요소간 배경색에 컬러를 주입해 구분감을 준다.