# UI
## Work tab:
![alt text](docs/res/image-work.png)

## Report tab:
![alt text](docs/res/image-report.png)

left 사이드바에 work / report를 선택할 수 있도록.
 - work 선택 시 placeholder / report 선택 시 하루 단위의 report 목록 조회 가능.
right 사이드바는 event list이다. 특정한 예를 들어 - '메신저 접속함' / '사내 게시판 접속' / '스프레드시트 on' 등의 이벤트 발생 시 log를 띄운다.

# background service - tracking notes
-> (나중에 config할 수 있도록...) 지금은 15s마다 note의 변경을 확인할 수 있도록 한다.
note 변경을, 업무 관점에서의 이벤트로 timestamp 포함해 정리한다.
