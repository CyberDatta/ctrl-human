# ctrl-human
Control Video Games with Your Body

# recreate environment instructions
  

# debug instructions
  cd ctrl_human
  pnpm tauri dev

# build instructions
Step 1 — Build the Python sidecars (one-time per machine, takes ~10 min):                                                                                        
  cd ctrl_human/src-python
  bash build_sidecars.sh                                                                                                                                           
                                                                                                                                                                   
Step 2 — Build the app:                                                                                                                                          
  cd ctrl_human                                                                                                                                                    
  pnpm tauri build        