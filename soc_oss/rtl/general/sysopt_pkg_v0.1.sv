// my_package.sv

`ifndef SYSOPT_PKG
    `define SYSOPT_PKG
    package sysopt_pkg; // package name

    typedef struct packed{
       bit [] 
    } sysopt_word;





    endpackage

    // import package in the design
    import sysopt_pkg::*;

`endif
 


type_opt_sce


optdatregiv
PMOPTPOS_SCE
PMOPTLEN_SCE


opt_sce.keyperm_enable

	sysopt_pkg::optdat optdatreg;

// optdatreg 
	optdatreg <= optdatregiv;
	optdatreg[optdatadr] <= optdatwr ? optdatwrdin: optdatreg[optdatadr];

// assign in system control 
	assign opt_sce = optdatreg[PMOPTPOS_SCE:PMOPT_SCE+PMOPTLEN_SCE];


// caller module
	sysopt_pkg::opt_sce opt_sce
	opt_sce.keyperm_enable



