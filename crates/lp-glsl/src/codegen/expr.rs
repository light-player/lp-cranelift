use cranelift_codegen::ir::{condcodes::{IntCC, FloatCC}, types, InstBuilder, Value};
use glsl::syntax::Expr;

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{infer_binary_result_type, infer_unary_result_type, promote_numeric, check_assignment, check_vector_constructor, is_vector_type_name};

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;

/// Component naming sets for vector swizzles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingSet {
    XYZW,  // Position/generic: x, y, z, w
    RGBA,  // Color: r, g, b, a
    STPQ,  // Texture coordinates: s, t, p, q
}

impl<'a> CodegenContext<'a> {
    /// Translate expression and return values (vec for vectors, single element for scalars) and type
    pub fn translate_expr_typed(&mut self, expr: &Expr) -> Result<(Vec<Value>, GlslType), String> {
        match expr {
            // Literals - scalars return single-element vec
            Expr::IntConst(n) => {
                let val = self.builder.ins().iconst(types::I32, *n as i64);
                Ok((vec![val], GlslType::Int))
            }

            Expr::FloatConst(f) => {
                let val = self.builder.ins().f32const(*f);
                Ok((vec![val], GlslType::Float))
            }

            Expr::BoolConst(b) => {
                let val = self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
                Ok((vec![val], GlslType::Bool))
            }

            // Variable reference - returns all components
            Expr::Variable(ident) => {
                let vars = self
                    .lookup_variables(&ident.0)
                    .ok_or_else(|| format!("Variable '{}' not found", ident.0))?
                    .to_vec(); // Clone to avoid borrow issues
                let ty = self
                    .lookup_variable_type(&ident.0)
                    .ok_or_else(|| format!("Variable type not found for '{}'", ident.0))?
                    .clone();
                
                let vals: Vec<Value> = vars.iter()
                    .map(|&v| self.builder.use_var(v))
                    .collect();
                
                Ok((vals, ty))
            }

            // Function calls - check if it's a type constructor or built-in
            Expr::FunCall(func_ident, args) => {
                // Extract identifier name from FunIdentifier enum
                let func_name = match func_ident {
                    glsl::syntax::FunIdentifier::Identifier(ident) => &ident.0,
                    _ => return Err("Complex function identifiers not yet supported".to_string()),
                };
                
                // Check if it's a type constructor
                if is_vector_type_name(func_name) {
                    return self.translate_vector_constructor(func_name, args);
                }
                
                // Check if it's a built-in function
                if crate::semantic::builtins::is_builtin_function(func_name) {
                    return self.translate_builtin_call_expr(func_name, args);
                }
                
                // User-defined function
                self.translate_user_function_call(func_name, args)
            }

            // Binary operators - both scalars and vectors
            Expr::Binary(op, lhs, rhs) => {
                let (lhs_vals, lhs_ty) = self.translate_expr_typed(lhs)?;
                let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

                // Check if it's vector or scalar operation
                if lhs_ty.is_vector() || rhs_ty.is_vector() {
                    self.translate_vector_binary_op(op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty)
                } else {
                    // Scalar operation
                    let lhs_val = lhs_vals[0];
                    let rhs_val = rhs_vals[0];

                    // Infer result type and validate
                    let result_ty = infer_binary_result_type(op, &lhs_ty, &rhs_ty)?;

                    // Promote operands to common type (for arithmetic ops)
                    let common_ty = promote_numeric(&lhs_ty, &rhs_ty);
                    let lhs_val = self.coerce_to_type(lhs_val, &lhs_ty, &common_ty)?;
                    let rhs_val = self.coerce_to_type(rhs_val, &rhs_ty, &common_ty)?;

                    // Generate operation
                    let result_val = self.translate_scalar_binary_op(op, lhs_val, rhs_val, &common_ty)?;
                    Ok((vec![result_val], result_ty))
                }
            }

            // Unary operators - scalars only for now
            Expr::Unary(op, expr) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                
                if vals.len() != 1 {
                    return Err("Vector unary ops not yet implemented".to_string());
                }

                let val = vals[0];
                let result_ty = infer_unary_result_type(op, &ty)?;
                let result_val = self.translate_unary_op(op, val, &ty)?;
                Ok((vec![result_val], result_ty))
            }

            // Component access (field selection) - supports multi-component swizzles
            Expr::Dot(expr, field) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                
                if !ty.is_vector() {
                    return Err(format!("Component access on non-vector type: {:?}", ty));
                }

                let indices = Self::parse_vector_swizzle(&field.0, &ty)?;
                let base_ty = ty.vector_base_type().unwrap();
                
                if indices.len() == 1 {
                    // Single component: return scalar
                    Ok((vec![vals[indices[0]]], base_ty))
                } else {
                    // Multi-component: return vector
                    let result_vals: Vec<Value> = indices.iter()
                        .map(|&idx| vals[idx])
                        .collect();
                    let result_ty = GlslType::vector_type(&base_ty, indices.len())
                        .ok_or_else(|| format!("Cannot create vector of size {}", indices.len()))?;
                    Ok((result_vals, result_ty))
                }
            }

            // Assignment
            Expr::Assignment(lhs, op, rhs) => self.translate_assignment_typed(lhs, op, rhs),

            _ => Err(format!("Expression not supported yet: {:?}", expr)),
        }
    }

    /// Legacy wrapper for compatibility - returns just the first value (for scalars)
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        let (vals, _ty) = self.translate_expr_typed(expr)?;
        vals.into_iter().next().ok_or_else(|| "Expression produced no values".to_string())
    }

    /// Translate scalar binary operation (works on single values)
    fn translate_scalar_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs: Value,
        rhs: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, String> {
        use glsl::syntax::BinaryOp::*;

        let val = match op {
            // Arithmetic operators - dispatch based on type
            Add => match operand_ty {
                GlslType::Int => self.builder.ins().iadd(lhs, rhs),
                GlslType::Float => self.builder.ins().fadd(lhs, rhs),
                _ => return Err(format!("Add not supported for {:?}", operand_ty)),
            },
            Sub => match operand_ty {
                GlslType::Int => self.builder.ins().isub(lhs, rhs),
                GlslType::Float => self.builder.ins().fsub(lhs, rhs),
                _ => return Err(format!("Sub not supported for {:?}", operand_ty)),
            },
            Mult => match operand_ty {
                GlslType::Int => self.builder.ins().imul(lhs, rhs),
                GlslType::Float => self.builder.ins().fmul(lhs, rhs),
                _ => return Err(format!("Mult not supported for {:?}", operand_ty)),
            },
            Div => match operand_ty {
                GlslType::Int => self.builder.ins().sdiv(lhs, rhs),
                GlslType::Float => self.builder.ins().fdiv(lhs, rhs),
                _ => return Err(format!("Div not supported for {:?}", operand_ty)),
            },

            // Comparison operators - dispatch based on type
            Equal => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
                _ => return Err(format!("Equal not supported for {:?}", operand_ty)),
            },
            NonEqual => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
                _ => return Err(format!("NonEqual not supported for {:?}", operand_ty)),
            },
            LT => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThan, lhs, rhs),
                _ => return Err(format!("LT not supported for {:?}", operand_ty)),
            },
            GT => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs),
                _ => return Err(format!("GT not supported for {:?}", operand_ty)),
            },
            LTE => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
                _ => return Err(format!("LTE not supported for {:?}", operand_ty)),
            },
            GTE => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
                _ => return Err(format!("GTE not supported for {:?}", operand_ty)),
            },

            // Logical operators (bool only, already validated by type_check)
            And | Or | Xor => {
                return Err(format!("Logical operator {:?} not yet implemented", op));
            }

            _ => return Err(format!("Binary operator not supported yet: {:?}", op)),
        };

        Ok(val)
    }

    /// Translate vector binary operation (component-wise or scalar-vector)
    fn translate_vector_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs_vals: Vec<Value>,
        lhs_ty: &GlslType,
        rhs_vals: Vec<Value>,
        rhs_ty: &GlslType,
    ) -> Result<(Vec<Value>, GlslType), String> {
        use glsl::syntax::BinaryOp::*;

        // Validate operation is allowed on vectors
        match op {
            Add | Sub | Mult | Div => {}, // allowed
            _ => return Err(format!("Operator {:?} not supported on vectors yet", op)),
        }

        // Determine result type and operation mode
        enum VectorOpMode {
            ComponentWise,    // vec + vec
            VectorScalar,     // vec + scalar
            ScalarVector,     // scalar + vec
        }

        let (result_ty, mode) = if lhs_ty.is_vector() && rhs_ty.is_vector() {
            // vec + vec: component-wise, types must match
            if lhs_ty != rhs_ty {
                return Err(format!(
                    "Vector operation requires matching types, got {:?} and {:?}",
                    lhs_ty, rhs_ty
                ));
            }
            (lhs_ty.clone(), VectorOpMode::ComponentWise)
        } else if lhs_ty.is_vector() && rhs_ty.is_scalar() {
            // vec + scalar: scalar applied to each component
            let vec_base = lhs_ty.vector_base_type().unwrap();
            // Check if scalar can be used with vector base type
            if !rhs_ty.is_numeric() || !vec_base.is_numeric() {
                return Err(format!(
                    "Cannot use {:?} with {:?}",
                    rhs_ty, lhs_ty
                ));
            }
            (lhs_ty.clone(), VectorOpMode::VectorScalar)
        } else if lhs_ty.is_scalar() && rhs_ty.is_vector() {
            // scalar + vec: scalar applied to each component
            let vec_base = rhs_ty.vector_base_type().unwrap();
            if !lhs_ty.is_numeric() || !vec_base.is_numeric() {
                return Err(format!(
                    "Cannot use {:?} with {:?}",
                    lhs_ty, rhs_ty
                ));
            }
            (rhs_ty.clone(), VectorOpMode::ScalarVector)
        } else {
            unreachable!("translate_vector_binary_op called with non-vector types");
        };

        let base_ty = result_ty.vector_base_type().unwrap();
        let component_count = result_ty.component_count().unwrap();

        let mut result_vals = Vec::new();
        
        match mode {
            VectorOpMode::ComponentWise => {
                // vec3(a,b,c) + vec3(d,e,f) = vec3(a+d, b+e, c+f)
                for i in 0..component_count {
                    let lhs_comp = lhs_vals[i];
                    let rhs_comp = rhs_vals[i];
                    let result_comp = self.translate_scalar_binary_op(op, lhs_comp, rhs_comp, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
            VectorOpMode::VectorScalar => {
                // vec3(a,b,c) * s = vec3(a*s, b*s, c*s)
                let scalar = rhs_vals[0];
                // Coerce scalar to vector base type if needed
                let scalar = self.coerce_to_type(scalar, rhs_ty, &base_ty)?;
                for &comp in &lhs_vals {
                    let result_comp = self.translate_scalar_binary_op(op, comp, scalar, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
            VectorOpMode::ScalarVector => {
                // s * vec3(a,b,c) = vec3(s*a, s*b, s*c)
                let scalar = lhs_vals[0];
                // Coerce scalar to vector base type if needed
                let scalar = self.coerce_to_type(scalar, lhs_ty, &base_ty)?;
                for &comp in &rhs_vals {
                    let result_comp = self.translate_scalar_binary_op(op, scalar, comp, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
        }

        Ok((result_vals, result_ty))
    }

    fn translate_unary_op(
        &mut self,
        op: &glsl::syntax::UnaryOp,
        val: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, String> {
        use glsl::syntax::UnaryOp::*;

        let result = match op {
            Minus => match operand_ty {
                GlslType::Int => self.builder.ins().ineg(val),
                GlslType::Float => self.builder.ins().fneg(val),
                _ => return Err(format!("Unary minus not supported for {:?}", operand_ty)),
            },
            Not => {
                if operand_ty != &GlslType::Bool {
                    return Err(format!("Logical NOT requires bool, got {:?}", operand_ty));
                }
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().icmp(IntCC::Equal, val, zero)
            }
            _ => return Err(format!("Unary operator not supported yet: {:?}", op)),
        };

        Ok(result)
    }

    /// Coerce a value from one type to another (implements GLSL implicit conversions)
    fn coerce_to_type(
        &mut self,
        val: Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
    ) -> Result<Value, String> {
        if from_ty == to_ty {
            return Ok(val);
        }

        match (from_ty, to_ty) {
            (GlslType::Int, GlslType::Float) => {
                // int → float: fcvt_from_sint
                Ok(self.builder.ins().fcvt_from_sint(types::F32, val))
            }
            _ => Err(format!("Cannot implicitly convert {:?} to {:?}", from_ty, to_ty)),
        }
    }

    fn translate_assignment_typed(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Only simple assignment (=) for now
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err("Only simple assignment (=) supported".to_string());
        }

        // Check if lhs is component access
        if let Expr::Dot(vec_expr, field) = lhs {
            return self.translate_component_assignment(vec_expr, field, rhs);
        }

        // Get variable name from lhs
        let var_name = match lhs {
            Expr::Variable(ident) => &ident.0,
            _ => return Err("Assignment lhs must be variable".to_string()),
        };

        let vars = self
            .lookup_variables(var_name)
            .ok_or_else(|| format!("Variable '{}' not found", var_name))?
            .to_vec(); // Clone to avoid borrow issues

        let lhs_ty = self
            .lookup_variable_type(var_name)
            .ok_or_else(|| format!("Variable type not found for '{}'", var_name))?
            .clone();

        // Translate RHS
        let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

        // Validate assignment (check implicit conversion is allowed)
        check_assignment(&lhs_ty, &rhs_ty)?;

        // Check component counts match
        if vars.len() != rhs_vals.len() {
            return Err(format!(
                "Component count mismatch in assignment: {} vs {}",
                vars.len(), rhs_vals.len()
            ));
        }

        // Coerce and assign each component
        let rhs_base = if rhs_ty.is_vector() {
            rhs_ty.vector_base_type().unwrap()
        } else {
            rhs_ty.clone()
        };
        let lhs_base = if lhs_ty.is_vector() {
            lhs_ty.vector_base_type().unwrap()
        } else {
            lhs_ty.clone()
        };

        let mut coerced_vals = Vec::new();
        for (var, val) in vars.iter().zip(&rhs_vals) {
            let coerced = self.coerce_to_type(*val, &rhs_base, &lhs_base)?;
            self.builder.def_var(*var, coerced);
            coerced_vals.push(coerced);
        }

        // Assignment expression has same type as LHS
        Ok((coerced_vals, lhs_ty))
    }

    fn translate_component_assignment(
        &mut self,
        vec_expr: &Expr,
        field: &glsl::syntax::Identifier,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Get variable name
        let var_name = match vec_expr {
            Expr::Variable(ident) => &ident.0,
            _ => return Err("Component assignment only supported on variables".to_string()),
        };

        let vars = self.lookup_variables(var_name)
            .ok_or_else(|| format!("Variable '{}' not found", var_name))?
            .to_vec(); // Clone to avoid borrow issues
        let vec_ty = self.lookup_variable_type(var_name)
            .ok_or_else(|| format!("Variable type not found for '{}'", var_name))?
            .clone();

        if !vec_ty.is_vector() {
            return Err(format!("Component access on non-vector variable: {}", var_name));
        }

        // Parse swizzle (supports multi-component assignment)
        let indices = Self::parse_vector_swizzle(&field.0, &vec_ty)?;
        let base_ty = vec_ty.vector_base_type().unwrap();

        // Check for duplicates (illegal in assignment LHS)
        if Self::has_duplicates(&indices) {
            return Err(format!("Swizzle '{}' contains duplicate components (illegal in assignment)", field.0));
        }

        // Translate RHS
        let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;
        
        // Validate sizes match
        if rhs_vals.len() != indices.len() {
            return Err(format!(
                "Swizzle assignment size mismatch: {} components on LHS, {} on RHS",
                indices.len(), rhs_vals.len()
            ));
        }

        // Type check base types
        let rhs_base = if rhs_ty.is_vector() {
            rhs_ty.vector_base_type().unwrap()
        } else {
            rhs_ty.clone()
        };
        check_assignment(&base_ty, &rhs_base)?;

        // Assign each component
        for (i, &idx) in indices.iter().enumerate() {
            let rhs_val = self.coerce_to_type(rhs_vals[i], &rhs_base, &base_ty)?;
            self.builder.def_var(vars[idx], rhs_val);
        }

        // Return all current values (read other components)
        let mut result_vals = Vec::new();
        for &var in &vars {
            result_vals.push(self.builder.use_var(var));
        }

        // Component assignment returns the whole vector
        Ok((result_vals, vec_ty))
    }

    fn translate_vector_constructor(
        &mut self,
        type_name: &str,
        args: &[Expr],
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Translate all arguments
        let mut arg_vals: Vec<Vec<Value>> = Vec::new();
        let mut arg_types: Vec<GlslType> = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            arg_vals.push(vals);
            arg_types.push(ty);
        }

        // Type check constructor
        let result_type = check_vector_constructor(type_name, &arg_types)?;
        let base_type = result_type.vector_base_type().unwrap();
        let component_count = result_type.component_count().unwrap();

        // Generate component values
        let mut components = Vec::new();

        // Case 1: Single scalar broadcast
        if arg_types.len() == 1 && arg_types[0].is_scalar() {
            let scalar = arg_vals[0][0];
            let coerced = self.coerce_to_type(scalar, &arg_types[0], &base_type)?;
            for _ in 0..component_count {
                components.push(coerced);
            }
        }
        // Case 2: Single vector conversion
        else if arg_types.len() == 1 && arg_types[0].is_vector() {
            let src_base = arg_types[0].vector_base_type().unwrap();
            for val in &arg_vals[0] {
                components.push(self.coerce_to_type(*val, &src_base, &base_type)?);
            }
        }
        // Case 3: Concatenation
        else {
            for (vals, ty) in arg_vals.iter().zip(&arg_types) {
                let arg_base = if ty.is_vector() {
                    ty.vector_base_type().unwrap()
                } else {
                    ty.clone()
                };
                
                for &val in vals {
                    components.push(self.coerce_to_type(val, &arg_base, &base_type)?);
                }
            }
        }

        Ok((components, result_type))
    }

    /// Translate built-in function call
    fn translate_builtin_call_expr(
        &mut self,
        name: &str,
        args: &[glsl::syntax::Expr],
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Translate all arguments
        let mut translated_args = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            translated_args.push((vals, ty));
        }
        
        // Delegate to built-in implementation
        self.translate_builtin_call(name, translated_args)
    }

    /// Parse vector component swizzle and return indices
    /// Supports xyzw, rgba, stpq naming sets
    /// Can parse multiple components: "xy", "rgba", "zyx", "xxxx", etc.
    fn parse_vector_swizzle(name: &str, vec_ty: &GlslType) -> Result<Vec<usize>, String> {
        if name.is_empty() {
            return Err("Empty swizzle".to_string());
        }
        
        if name.len() > 4 {
            return Err(format!("Swizzle can have at most 4 components, got {}", name.len()));
        }
        
        let component_count = vec_ty.component_count()
            .ok_or_else(|| format!("{:?} is not a vector type", vec_ty))?;
        
        // Determine which naming set is used and validate consistency
        let naming_set = Self::determine_naming_set(name)?;
        
        // Parse each character
        let mut indices = Vec::new();
        for ch in name.chars() {
            let idx = Self::parse_single_component(ch, naming_set)?;
            
            // Validate index is within bounds
            if idx >= component_count {
                return Err(format!(
                    "Component '{}' not valid for {:?} (has only {} components)",
                    ch, vec_ty, component_count
                ));
            }
            
            indices.push(idx);
        }
        
        Ok(indices)
    }

    /// Determine which naming set is used in a swizzle and validate consistency
    fn determine_naming_set(swizzle: &str) -> Result<NamingSet, String> {
        let mut xyzw_count = 0;
        let mut rgba_count = 0;
        let mut stpq_count = 0;
        
        for ch in swizzle.chars() {
            match ch {
                'x' | 'y' | 'z' | 'w' => xyzw_count += 1,
                'r' | 'g' | 'b' | 'a' => rgba_count += 1,
                's' | 't' | 'p' | 'q' => stpq_count += 1,
                _ => return Err(format!("Invalid swizzle character: '{}'", ch)),
            }
        }
        
        let sets_used = (xyzw_count > 0) as u32 + (rgba_count > 0) as u32 + (stpq_count > 0) as u32;
        
        if sets_used > 1 {
            return Err(format!("Swizzle '{}' mixes component naming sets (xyzw/rgba/stpq)", swizzle));
        }
        
        if xyzw_count > 0 { 
            Ok(NamingSet::XYZW) 
        } else if rgba_count > 0 { 
            Ok(NamingSet::RGBA) 
        } else { 
            Ok(NamingSet::STPQ) 
        }
    }

    /// Parse a single component character given a naming set
    fn parse_single_component(ch: char, naming_set: NamingSet) -> Result<usize, String> {
        match naming_set {
            NamingSet::XYZW => match ch {
                'x' => Ok(0),
                'y' => Ok(1),
                'z' => Ok(2),
                'w' => Ok(3),
                _ => Err(format!("Invalid component '{}' for xyzw naming set", ch)),
            },
            NamingSet::RGBA => match ch {
                'r' => Ok(0),
                'g' => Ok(1),
                'b' => Ok(2),
                'a' => Ok(3),
                _ => Err(format!("Invalid component '{}' for rgba naming set", ch)),
            },
            NamingSet::STPQ => match ch {
                's' => Ok(0),
                't' => Ok(1),
                'p' => Ok(2),
                'q' => Ok(3),
                _ => Err(format!("Invalid component '{}' for stpq naming set", ch)),
            },
        }
    }

    /// Check if a slice of indices contains duplicates
    fn has_duplicates(indices: &[usize]) -> bool {
        let mut seen = HashSet::new();
        for &idx in indices {
            if !seen.insert(idx) {
                return true;
            }
        }
        false
    }
}

