module AES_flat_model(
    // input logic [127:0] plaintext,   //128-bit plaintext input
    // input logic [127:0] key,         //128-bit key input for AES-128 (for AES-256, this would be 256 bits)
    // output logic [127:0] ciphertext,  // 128-bit encrypted output
    // input logic  clk,
    // input logic rst_n,
    // input logic start,
    // output logic done                 //Signal indicating encryption completion

);
    logic [127:0] plaintext;   //128-bit plaintext input
    logic [127:0] key;        //128-bit key input for AES-128 (for AES-256, this would be 256 bits)
    logic [127:0] ciphertext;  // 128-bit encrypted output

    logic  clk;
    logic rst_n;
    logic start;
    logic done;                 //Signal indicating encryption completion

    logic [0:15][7:0] state; //State Matrix
    logic [0:15][7:0] cipher_key;
    logic [0:175][7:0] round_key;
    logic [0:9][7:0] rcon;

    logic [0:15][7:0] IV; // Initial Vector

// S-Box definition
    logic [7:0] s_box [0:255] = {

        8'h63, 8'h7c, 8'h77, 8'h7b, 8'hf2, 8'h6b, 8'h6f, 8'hc5, 8'h30, 8'h01, 8'h67, 8'h2b, 8'hfe, 8'hd7, 8'hab, 8'h76,
        8'hca, 8'h82, 8'hc9, 8'h7d, 8'hfa, 8'h59, 8'h47, 8'hf0, 8'had, 8'hd4, 8'ha2, 8'haf, 8'h9c, 8'ha4, 8'h72, 8'hc0,
        8'hb7, 8'hfd, 8'h93, 8'h26, 8'h36, 8'h3f, 8'hf7, 8'hcc, 8'h34, 8'ha5, 8'he5, 8'hf1, 8'h71, 8'hd8, 8'h31, 8'h15,
        8'h04, 8'hc7, 8'h23, 8'hc3, 8'h18, 8'h96, 8'h05, 8'h9a, 8'h07, 8'h12, 8'h80, 8'he2, 8'heb, 8'h27, 8'hb2, 8'h75,
        8'h09, 8'h83, 8'h2c, 8'h1a, 8'h1b, 8'h6e, 8'h5a, 8'ha0, 8'h52, 8'h3b, 8'hd6, 8'hb3, 8'h29, 8'he3, 8'h2f, 8'h84,
        8'h53, 8'hd1, 8'h00, 8'hed, 8'h20, 8'hfc, 8'hb1, 8'h5b, 8'h6a, 8'hcb, 8'hbe, 8'h39, 8'h4a, 8'h4c, 8'h58, 8'hcf,
        8'hd0, 8'hef, 8'haa, 8'hfb, 8'h43, 8'h4d, 8'h33, 8'h85, 8'h45, 8'hf9, 8'h02, 8'h7f, 8'h50, 8'h3c, 8'h9f, 8'ha8,
        8'h51, 8'ha3, 8'h40, 8'h8f, 8'h92, 8'h9d, 8'h38, 8'hf5, 8'hbc, 8'hb6, 8'hda, 8'h21, 8'h10, 8'hff, 8'hf3, 8'hd2,
        8'hcd, 8'h0c, 8'h13, 8'hec, 8'h5f, 8'h97, 8'h44, 8'h17, 8'hc4, 8'ha7, 8'h7e, 8'h3d, 8'h64, 8'h5d, 8'h19, 8'h73,
        8'h60, 8'h81, 8'h4f, 8'hdc, 8'h22, 8'h2a, 8'h90, 8'h88, 8'h46, 8'hee, 8'hb8, 8'h14, 8'hde, 8'h5e, 8'h0b, 8'hdb,
        8'he0, 8'h32, 8'h3a, 8'h0a, 8'h49, 8'h06, 8'h24, 8'h5c, 8'hc2, 8'hd3, 8'hac, 8'h62, 8'h91, 8'h95, 8'he4, 8'h79,
        8'he7, 8'hc8, 8'h37, 8'h6d, 8'h8d, 8'hd5, 8'h4e, 8'ha9, 8'h6c, 8'h56, 8'hf4, 8'hea, 8'h65, 8'h7a, 8'hae, 8'h08,
        8'hba, 8'h78, 8'h25, 8'h2e, 8'h1c, 8'ha6, 8'hb4, 8'hc6, 8'he8, 8'hdd, 8'h74, 8'h1f, 8'h4b, 8'hbd, 8'h8b, 8'h8a,
        8'h70, 8'h3e, 8'hb5, 8'h66, 8'h48, 8'h03, 8'hf6, 8'h0e, 8'h61, 8'h35, 8'h57, 8'hb9, 8'h86, 8'hc1, 8'h1d, 8'h9e,
        8'he1, 8'hf8, 8'h98, 8'h11, 8'h69, 8'hd9, 8'h8e, 8'h94, 8'h9b, 8'h1e, 8'h87, 8'he9, 8'hce, 8'h55, 8'h28, 8'hdf,
        8'h8c, 8'ha1, 8'h89, 8'h0d, 8'hbf, 8'h36, 8'h42, 8'h68, 8'h41, 8'h99, 8'h2d, 8'h0f, 8'hb0, 8'h54, 8'hbb, 8'h16
};

// Inv-S-Box definition
logic [7:0] inv_s_box[0:255] = {

        8'h52, 8'h09, 8'h6A, 8'hD5, 8'h30, 8'h36, 8'hA5, 8'h38, 8'hBF, 8'h40, 8'hA3, 8'h9E, 8'h81, 8'hF3, 8'hD7, 8'hFB,
        8'h7C, 8'hE3, 8'h39, 8'h82, 8'h9B, 8'h2F, 8'hFF, 8'h87, 8'h34, 8'h8E, 8'h43, 8'h44, 8'hC4, 8'hDE, 8'hE9, 8'hCB,
        8'h54, 8'h7B, 8'h94, 8'h32, 8'hA6, 8'hC2, 8'h23, 8'h3D, 8'hEE, 8'h4C, 8'h95, 8'h0B, 8'h42, 8'hFA, 8'hC3, 8'h4E,
        8'h08, 8'h2E, 8'hA1, 8'h66, 8'h28, 8'hD9, 8'h24, 8'hB2, 8'h76, 8'h5B, 8'hA2, 8'h49, 8'h6D, 8'h8B, 8'hD1, 8'h25,
        8'h72, 8'hF8, 8'hF6, 8'h64, 8'h86, 8'h68, 8'h98, 8'h16, 8'hD4, 8'hA4, 8'h5C, 8'hCC, 8'h5D, 8'h65, 8'hB6, 8'h92,
        8'h6C, 8'h70, 8'h48, 8'h50, 8'hFD, 8'hED, 8'hB9, 8'hDA, 8'h5E, 8'h15, 8'h46, 8'h57, 8'hA7, 8'h8D, 8'h9D, 8'h84,
        8'h90, 8'hD8, 8'hAB, 8'h00, 8'h8C, 8'hBC, 8'hD3, 8'h0A, 8'hF7, 8'hE4, 8'h58, 8'h05, 8'hB8, 8'hB3, 8'h45, 8'h06,
        8'hD0, 8'h2C, 8'h1E, 8'h8F, 8'hCA, 8'h3F, 8'h0F, 8'h02, 8'hC1, 8'hAF, 8'hBD, 8'h03, 8'h01, 8'h13, 8'h8A, 8'h6B,
        8'h3A, 8'h91, 8'h11, 8'h41, 8'h4F, 8'h67, 8'hDC, 8'hEA, 8'h97, 8'hF2, 8'hCF, 8'hCE, 8'hF0, 8'hB4, 8'hE6, 8'h73,
        8'h96, 8'hAC, 8'h74, 8'h22, 8'hE7, 8'hAD, 8'h35, 8'h85, 8'hE2, 8'hF9, 8'h37, 8'hE8, 8'h1C, 8'h75, 8'hDF, 8'h6E,
        8'h47, 8'hF1, 8'h1A, 8'h71, 8'h1D, 8'h29, 8'hC5, 8'h89, 8'h6F, 8'hB7, 8'h62, 8'h0E, 8'hAA, 8'h18, 8'hBE, 8'h1B,
        8'hFC, 8'h56, 8'h3E, 8'h4B, 8'hC6, 8'hD2, 8'h79, 8'h20, 8'h9A, 8'hDB, 8'hC0, 8'hFE, 8'h78, 8'hCD, 8'h5A, 8'hF4,
        8'h1F, 8'hDD, 8'hA8, 8'h33, 8'h88, 8'h07, 8'hC7, 8'h31, 8'hB1, 8'h12, 8'h10, 8'h59, 8'h27, 8'h80, 8'hEC, 8'h5F,
        8'h60, 8'h51, 8'h7F, 8'hA9, 8'h19, 8'hB5, 8'h4A, 8'h0D, 8'h2D, 8'hE5, 8'h7A, 8'h9F, 8'h93, 8'hC9, 8'h9C, 8'hEF,
        8'hA0, 8'hE0, 8'h3B, 8'h4D, 8'hAE, 8'h2A, 8'hF5, 8'hB0, 8'hC8, 8'hEB, 8'hBB, 8'h3C, 8'h83, 8'h53, 8'h99, 8'h61,
        8'h17, 8'h2B, 8'h04, 8'h7E, 8'hBA, 8'h77, 8'hD6, 8'h26, 8'hE1, 8'h69, 8'h14, 8'h63, 8'h55, 8'h21, 8'h0C, 8'h7D
};



function void SubBytes();

    for (int i = 0; i < 16; i++) begin
        // Use state value as index directly for the s_box
        state[i] = s_box[state[i]];
    end

endfunction

function void Inv_SubBytes();

    for (int i = 0; i < 16; i++) begin
        // Use state value as index directly for the s_box
        state[i] = inv_s_box[state[i]];
    end

endfunction


function void ShiftRows();
//$display("State before shift rows: %x", state);
    // Row 0 remains unchanged

    //Row 1: shift left by 1 byte
        {state[1], state[5], state[9], state[13]} = {state[5],state[9],state[13],state[1]};

    //Row 2: shift left by 2 bytes
        {state[2], state[6], state[10], state[14]} = {state[10],state[14],state[2],state[6]};

    //Row 3: shift left by 3 bytes
        {state[3], state[7], state[11], state[15]} = {state[15],state[3],state[7],state[11]};
    //$display("State after shiftrows: %x", state);
endfunction


function void Inv_ShiftRows();
    reg [7:0] temp[3:0];
//$display("State before shift rows: %x", state);
    // Row 0 remains unchanged

    //Row 1: shift right by 1 byte
             {state[1], state[5], state[9], state[13]} = {state[13],state[1],state[5],state[9]};

    //Row 2: shift right by 2 bytes
           {state[2], state[6], state[10], state[14]} = {state[10],state[14],state[2],state[6]};

    //Row 3: shift right by 3 bytes
            {state[3], state[7], state[11], state[15]} = {state[7],state[11],state[15],state[3]};
    //$display("State after shiftrows: %x", state);
endfunction


function void MixColumns();
    reg [7:0] tmp, tm, t;
    reg [7:0]  col[3:0];

    for(int i = 0; i< 4; i++) begin
        // Copy the current column
        col[3:0] = {state[4*i+3], state[4*i+2], state[4*i+1], state[4*i]};

        // Apply the MixColumns operation
        t = col[0];
        tmp = col[0] ^ col[1] ^ col[2] ^ col[3];
        tm = col[0] ^ col[1]; tm = xtime(tm); col[0] ^= tm ^ tmp;
        tm = col[1] ^ col[2]; tm = xtime(tm); col[1] ^= tm ^ tmp;
        tm = col[2] ^ col[3]; tm = xtime(tm); col[2] ^= tm ^ tmp;
        tm = col[3] ^ t; tm = xtime(tm); col[3] ^= tm ^ tmp;

        //Store the result back into the state
         state[4*i+3] = col[3];
         state[4*i+2] = col[2];
         state[4*i+1] = col[1];
         state[4*i] = col[0];

    end
endfunction


function logic [7:0] xtime(logic [7:0] a);
    if (a[7]) // check if the leftmost bit is 1
        return {a[6:0], 1'b0} ^ 8'h1B; // shift left and XOR with 0x1B
    else
        return {a[6:0], 1'b0}; // simply shift left
endfunction

function void Inv_MixColumns();
    logic [7:0] u, v;
  //  state = 16'hE9F74EEC023020F61BF2CCF2353C21C7; // provided initial state

    for(int i = 0; i < 4; i++) begin
        u = xtime(xtime(state[4*i] ^ state[4*i + 2]));
      //  $display("u %x",u );
        v = xtime(xtime(state[4*i + 1] ^ state[4*i + 3]));
   //  $display("u %x",v);

    //  $display("state[4*i] before %x %x %x %x",state[4*i], state[4*i +1],state[4*i+2],state[4*i+3]  );
        state[4*i] = state[4*i] ^ u;

        state[4*i + 1] = state[4*i + 1] ^ v;
        state[4*i + 2] = state[4*i + 2] ^ u;
        state[4*i + 3] = state[4*i + 3] ^ v;
        //    $display("state[4*i] before %x %x %x %x",state[4*i],state[4*i+1],state[4*i+2],state[4*i+3] );
    end

    // You can then print the state to verify it matches the expected result:
    MixColumns();

endfunction



function void AddRoundKey(int round);

    for (int i = 0; i < 16; i++) begin
   //$display("State before AddRoundKey %x: %x", state[i],round_key[round*16 + i]);
        state[i] = state[i] ^ round_key[round*16 + i];
        //   $display("State after AddRoundKey  : %x", state);
    end

endfunction


function void KeyExpansion();
    int i;
    reg [7:0] temp[3:0];  // Temporary storage for a word.
    reg [7:0] rcon[11] = {8'h00,8'h01, 8'h02, 8'h04, 8'h08, 8'h10, 8'h20, 8'h40, 8'h80, 8'h1b, 8'h36};

    // The first round key is the key itself.
    for (i = 0; i < 16; i++) begin
        round_key[i] = cipher_key[i];
    end

    // ALL other round keys are found from the previous round keys
    for (i = 16; i < 176; i+=4) begin

         {temp[0],temp[1],temp[2],temp[3]} = {round_key[i-4],round_key[i-3],round_key[i-2],round_key[i-1]};

        // Every fourth word has a more complex method
        if ((i/4) % 4 == 0)
        begin
            // Rotate word
                 {temp[0],temp[1],temp[2],temp[3]} = {temp[1],temp[2],temp[3],temp[0]};

            // Apply SubBytes to each byte of the word
                {temp[0],temp[1],temp[2],temp[3]} =  { s_box[temp[0]],s_box[temp[1]],s_box[temp[2]],s_box[temp[3]]};

            //XOR with RCON
                temp[0] = temp[0] ^ rcon[i/16];

        end
        // Store the expanded key back
        round_key[i]   = round_key[i-16] ^ temp[0];
        round_key[i+1] = round_key[i-15] ^ temp[1];
        round_key[i+2] = round_key[i-14] ^ temp[2];
        round_key[i+3] = round_key[i-13] ^ temp[3];
    end
endfunction

function void CBC();
    $display("CBC mode\n");
    for (int i =0; i<16;i ++) begin
        state[i] = state[i] ^ IV[i];
    end
endfunction


initial
begin
    $fsdbDumpfile("aesref.fsdb");
    $fsdbDumpvars;
end

initial begin
    // Initialization
    // Define a state and cipher_key for illustration purposes
  state[0:15] = {
        8'hff, 8'h11, 8'h22, 8'h33,
        8'h44, 8'h55, 8'h66, 8'h77,
        8'h88, 8'h99, 8'haa, 8'hbb,
        8'hcc, 8'hdd, 8'hee, 8'hff

    };
   cipher_key[0:15]  = {
        8'h00, 8'h01, 8'h02, 8'h03,
        8'h04, 8'h05, 8'h06, 8'h07,
        8'h08, 8'h09, 8'h0A, 8'h0B,
        8'h0C, 8'h0D, 8'h0E, 8'h0F

    };

    IV[0:15] = {
        8'h11,8'h11,8'h11,8'h11,
        8'h22,8'h22,8'h22,8'h22,
        8'h33,8'h33,8'h33,8'h33,
        8'h44,8'h44,8'h44,8'h44
    };
//        CBC();
        KeyExpansion(); //Call this first to genertae all round keys
            //    for (int i = 0; i< 176; i++) begin
            //          $display("round_key[%d] = %x  ", i, round_key[i]);
            //      end
        AddRoundKey(0);    // Initial round key addition

        // Main rounds
        for(int round = 1; round < 10; round++) begin
            $display("Round: %d", round);
            $display("initial state: %x", state);

            SubBytes();
            $display("State after SubBytes:    %x", state);

            ShiftRows();
            $display("State after ShiftRows:   %x", state);

            MixColumns();
            $display("State after MixColumns:  %x", state);

            AddRoundKey(round);
            $display("State after AddRoundKey: %x", state);

        end

        //Final round
        SubBytes();
        ShiftRows();
        AddRoundKey(10);

    for (int i = 0; i < 16; i = i + 1)
    begin
        ciphertext[i*8 +: 8] = state[15-i];
    end
        $display("Cipher Text: %0h", ciphertext);

    end


    initial begin
    // Initialization
    // Define a state and cipher_key for illustration purposes
    state[0:15] = {
        8'h69, 8'hc4, 8'he0, 8'hd8,
        8'h6a, 8'h7b, 8'h04, 8'h30,
        8'hd8, 8'hcd, 8'hb7, 8'h80,
        8'h70, 8'hb4, 8'hc5, 8'h5a
        };

//CBC input
    state[0:15] = {
        8'h68, 8'h49, 8'h47, 8'hd5,
        8'h6f, 8'hd7, 8'heb, 8'hf7,
        8'h68, 8'hcb, 8'hb3, 8'h5e,
        8'hbe, 8'hce, 8'ha6, 8'h95
        };

    cipher_key[0:15]  = {
        8'h00, 8'h01, 8'h02, 8'h03,
        8'h04, 8'h05, 8'h06, 8'h07,
        8'h08, 8'h09, 8'h0A, 8'h0B,
        8'h0C, 8'h0D, 8'h0E, 8'h0F

    };
    $display("initial state: %x", state);
        KeyExpansion(); //Call this first to genertae all round keys
            //    for (int i = 0; i< 176; i++) begin
            //          $display("round_key[%d] = %x  ", i, round_key[i]);
            //      end

        AddRoundKey(10);    // Initial round key addition
        Inv_ShiftRows();
        Inv_SubBytes();

        // Main rounds
        for(int round = 9; round >0; round--) begin
            $display("Round: %d", round);
            $display("initial state: %x", state);
             AddRoundKey(round);
            $display("State after AddRoundKey:    %x", state);

            Inv_MixColumns();
            $display("State after inv MixColumns: %x", state);

            Inv_ShiftRows();
            $display("State after inv ShiftRows:  %x", state);

            Inv_SubBytes();
            $display("State after inv SubBytes:   %x", state);

        end


        AddRoundKey(0);
   CBC();
    for (int i = 0; i < 16; i = i + 1)
    begin
        plaintext[i*8 +: 8] = state[15-i];
    end
        $display("Recovered Text: %0h", plaintext);

    end
endmodule





