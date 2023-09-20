# week8 halo2 simple example

https://erroldrummond.gitbook.io/halo2-tutorial/section-3/conclusion
https://youtu.be/2FngWNTvjzk?si=GrxZ5-gc3jrv5jr9

https://github.com/EDGDrummond/EF-grant



이게 주요 프로그레스임. 
일단 plonk란 걸 쓴다는 가정하에
1. config 만들고 (모든 컬럼을 포함하는 )(랩핑))
2. chip 만들고. 이전에 만든 config를 포함함(랩핑)
3. 컴포저라고 불르는 trait를 만들어줌. 게이트를 얹기 위해..?
To recap, the steps were:
Decide what functionality you want in the chip, and thus pick an equation that allows you to provide that functionality (for now we are just using the original plonk equation).
Create a struct _Config that will wrap up all the columns you will need in your chip, and define their types.
Wrap the previous struct up with _marker: PhantomData in something generically named _Chip.
Create a trait to be implemented on your chip (can simply be called _Composer), putting all the gate types you will want to see as functions. Include any other functionality you will want on your chip that are algebraically possible (based on the equation you chose).

요게 중요하다고 합니다.
What we do want you to recall from this lab is the steps:
Define your config
Wrap your config with some marker in a struct (and build a new() for it)
Define a trait that will contain the functionality you want in your chip, thinking carefully about what each function (each function will usually represent a gate, except for example copy) should have as input or output
Implement those composer functions on your chip
