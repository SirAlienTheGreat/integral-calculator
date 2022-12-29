use std::f64;

pub(crate) fn calculate(input:&String) -> Result<f64, Box<dyn std::error::Error>>{    
    //Parentheses stuff
    //let input_chars = &input.chars().collect::<Vec<_>>();
    let mut stage = 1;
    let mut parentheses_to_ignore = 0;
    let mut first_parentheses_seen = false;

    let mut parentheses_string = "".to_string();
    let mut nonparentheses_string = input.clone();
    
    let input_chars = &nonparentheses_string.chars().collect::<Vec<_>>();

    for i in 0..input_chars.len(){ // Every character
        if input_chars[i] == '('{
            if !first_parentheses_seen{ // Big parentheses
                first_parentheses_seen = true;
                //parentheses_string.push(input_chars[i]);
                stage = 2;
            } else{ //Nested parentheses
                parentheses_to_ignore+=1;
                parentheses_string.push(input_chars[i]);
            }
        } else if input_chars[i] == ')' && stage == 2{
            if parentheses_to_ignore == 0 { // Big parentheses
                nonparentheses_string = nonparentheses_string.replace(&["(",&parentheses_string as &str,")"].join(""),&calculate(&parentheses_string)?.to_string());
                parentheses_string = "".to_string();
                first_parentheses_seen = false;
                stage = 1; 
            } else{ //Nested parentheses
                parentheses_to_ignore+=-1;
                parentheses_string.push(input_chars[i]);
            }
        } else if stage == 2{ //Things inside big parentheses
            parentheses_string.push(input_chars[i]);
        }
    }
    
    
    let inputo = &nonparentheses_string;
    
    //Actual Calculation
    let operators = "+*/^";
    let mut terms_string:Vec<String>= Vec::new();
    let mut terms_float:Vec<f64>= Vec::new();
    let mut operator_after:Vec<char>= Vec::new();
    let mut key = 0;

    // IE 4+9 would be converted to:
        //  terms_float: [4,9]
        //operator_after [+]

    for i in inputo.chars() { //every character
        if operators.contains(i){ //if i is an operator
            operator_after.push(*&i);
            key+=1;
        } else if terms_string.len()<=key{ //if i is the first character in the term
            terms_string.push(i.to_string());
        } else{ //if i the second (or later) character in the term
            terms_string[key].push(*&i);
        }
    }

    for i in &terms_string{ //For every term
        //convert strings to float
        //intercept trig functions
        if i.len()>=4{ //if term is long enough to be a trig function
            let i_chars = &i.chars().collect::<Vec<_>>();
            // Inverse trig functions
            if i_chars[0] == 'a' && i_chars[1] == 's' && i_chars[2] == 'i'&& i_chars[3] == 'n'{ //asin
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y).asin()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            }else if i_chars[0] == 'a' && i_chars[1] == 'c' && i_chars[2] == 'o'&& i_chars[3] == 's'{ //acos
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y).acos()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            } else if i_chars[0] == 'a' && i_chars[1] == 't' && i_chars[2] == 'a'&& i_chars[3] == 'n'{//atan
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y).atan()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            } else if i_chars[0] == 'a' && i_chars[1] == 'c' && i_chars[2] == 's'&& i_chars[3] == 'c'{ //acsc
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y.powf(-1.0)).asin()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            }else if i_chars[0] == 'a' && i_chars[1] == 's' && i_chars[2] == 'e'&& i_chars[3] == 'c'{//asec
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y.powf(-1.0)).acos()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            } else if i_chars[0] == 'a' && i_chars[1] == 'c' && i_chars[2] == 'o'&& i_chars[3] == 't'{//acot
                let y:f64 = i_chars[4..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y.powf(-1.0)).atan()*180.0/(std::f64::consts::PI)*10000.0).round()/10000.0);
            
            //Trig functions
            } else if i_chars[0] == 's' && i_chars[1] == 'i' && i_chars[2] == 'n'{//sin
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y*std::f64::consts::PI/180.0).sin()*10000.0).round()/10000.0);
            } else if i_chars[0] == 'c' && i_chars[1] == 'o' && i_chars[2] == 's'{//cos
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y*std::f64::consts::PI/180.0).cos()*10000.0).round()/10000.0);
            } else if i_chars[0] == 't' && i_chars[1] == 'a' && i_chars[2] == 'n'{//tan
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push(((y*std::f64::consts::PI/180.0).tan()*10000.0).round()/10000.0);
            } else if i_chars[0] == 'c' && i_chars[1] == 's' && i_chars[2] == 'c'{//csc
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push((((y*std::f64::consts::PI/180.0).sin()).powf(-1.0)*10000.0).round()/10000.0);
            } else if i_chars[0] == 's' && i_chars[1] == 'e' && i_chars[2] == 'c'{//sec
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push((((y*std::f64::consts::PI/180.0).cos()).powf(-1.0)*10000.0).round()/10000.0);
            } else if i_chars[0] == 'c' && i_chars[1] == 'o' && i_chars[2] == 't'{//cot
                let y:f64 = i_chars[3..].iter().collect::<String>().parse::<f64>()?;
                terms_float.push((((y*std::f64::consts::PI/180.0).tan()).powf(-1.0)*10000.0).round()/10000.0);
            }else{
                let y = i.replace("(","").replace(")","");
                let x: f64 = y.parse()?;
                terms_float.push(x);
            }
        } else{
            let y = i.replace("(","").replace(")","");
            let x: f64 = y.parse()?;
            terms_float.push(x);
        }
    } 
    
    let mut x:f64 = 0.0;
    while x <operator_after.len() as f64  { //Exponents
        let i = x as usize;
        if operator_after[i]=='^'{
            terms_float[i] = terms_float[i].powf(terms_float[i+1]);
            terms_float.remove(i+1);
            operator_after.remove(i);
        }else{
            x += 1.0;
        }
    }

    x = 0.0;
    while x <(terms_float.len()-1) as f64  { //Multiplication and division
        let i = x as usize;
        if operator_after[i]=='*'{
            terms_float[i] = terms_float[i]*terms_float[i+1];
            terms_float.remove(i+1);
            operator_after.remove(i);
        }else if operator_after[i]=='/' {
            terms_float[i] = terms_float[i] / terms_float[i + 1];
            terms_float.remove(i + 1);
            operator_after.remove(i);
        } else{
            x += 1.0;
        }
    }

    x = 0.0;
    while x <((terms_float.len() as f64)-1.0)  { //Addition and Subtraction
        let i = x as usize;
        if operator_after[i]=='+'{
            terms_float[i] = terms_float[i]+terms_float[i+1];
            terms_float.remove(i+1);
            operator_after.remove(i);
        }else if operator_after[i]=='-'{
            terms_float[i] = terms_float[i]-terms_float[i+1];
            terms_float.remove(i+1);
            operator_after.remove(i);
        }else{
            x += 1.0;
        }
    }
    let output = terms_float[0];
    return Ok(output); 
}