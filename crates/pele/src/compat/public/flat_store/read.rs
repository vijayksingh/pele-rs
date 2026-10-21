use super::super::{
    captured_admission::{
        AdmissionKind, AdmissionRead, Extent, Header, ScalarAtomRef, ScalarPropertyRef,
    },
    captured_read::{CapturedRead, NodeHeader, NodeKind, PropertyPrefix, PROPERTY_CHUNK},
};
use super::*;
impl Data {
    fn read_header(&self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        if id as usize >= self.headers.len() {
            return Ok(None);
        }
        let h = self.headers.get(id as usize)?;
        if h[0] == u32::MAX {
            return Ok(None);
        }
        let kind = match h[0] {
            0 => NodeKind::Object,
            1 => NodeKind::Instance,
            2 => NodeKind::Array,
            3 => NodeKind::Map,
            4 => NodeKind::Set,
            5 => NodeKind::Date,
            6 => NodeKind::Error,
            _ => return Err(fault()),
        };
        // A malformed scalar is never handed to the importer as a valid Array.
        if (h[0] == 2 || h[0] == 6) && h[2] != 0 {
            return Err(fault());
        }
        if h[0] == 6 {
            error_kind(h[1])?;
        }
        Ok(Some(NodeHeader {
            kind,
            property_count: h[4],
            array_length: if h[0] == 2 { Some(h[1]) } else { None },
        }))
    }
    fn admission_header(&self, id: usize) -> Result<Header, PeleError> {
        let h = self.headers.get(id)?;
        let kind = match h[0] {
            0 => AdmissionKind::Object,
            1 => AdmissionKind::Instance(PrototypeId(h[1])),
            2 if h[2] != 0 => AdmissionKind::InvalidArray,
            2 => AdmissionKind::Array(h[1]),
            3 => AdmissionKind::Map(h[6] as usize),
            4 => AdmissionKind::Set(h[6] as usize),
            5 => AdmissionKind::Date(h[1] as u64 | ((h[2] as u64) << 32)),
            6 if h[2] != 0 => AdmissionKind::InvalidError,
            6 => AdmissionKind::Error(error_kind(h[1])?),
            _ => return Err(fault()),
        };
        Ok(Header {
            kind,
            properties: h[4] as usize,
        })
    }
    fn property_range(&self, id: usize, start: usize) -> Result<(usize, usize), PeleError> {
        let h = self.headers.get(id)?;
        if h[0] == u32::MAX || start > h[4] as usize {
            return Err(fault());
        }
        Ok((h[3] as usize + start, h[4] as usize - start))
    }
    fn items_range(&self, id: usize, start: usize, kind: u32) -> Result<(usize, usize), PeleError> {
        let h = self.headers.get(id)?;
        if h[0] != kind || start > h[6] as usize {
            return Err(fault());
        }
        Ok((
            h[5] as usize + start * if kind == 3 { 2 } else { 1 },
            h[6] as usize - start,
        ))
    }
    fn own_fields(
        &self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Atom>],
    ) -> Result<(), PeleError> {
        if names.len() != out.len() {
            return Err(fault());
        }
        out.fill(None);
        let mut remaining = names.len();
        if remaining == 0 || id as usize >= self.headers.len() {
            return Ok(());
        }
        let h = self.headers.get(id as usize)?;
        if h[0] == u32::MAX {
            return Ok(());
        }
        for i in 0..h[4] as usize {
            let p = self.properties.get(h[3] as usize + i)?;
            for (name, value) in names.iter().zip(out.iter_mut()) {
                if value.is_none() && self.strings.is_text(p[0], name)? {
                    *value = Some(atom([p[2], p[3], p[4]])?);
                    remaining -= 1;
                }
            }
            if remaining == 0 {
                break;
            }
        }
        Ok(())
    }
    fn prefixes(
        &self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Atom>],
    ) -> Result<usize, PeleError> {
        for slot in out.iter_mut() {
            *slot = PropertyPrefix::default();
        }
        let (start, remaining) = self.property_range(id as usize, start as usize)?;
        let count = remaining.min(out.len()).min(PROPERTY_CHUNK);
        for (i, slot) in out.iter_mut().take(count).enumerate() {
            let p = self.properties.get(start + i)?;
            let [key, len] = self.strings.descriptors.get(p[0] as usize)?;
            slot.key_units = len;
            for j in 0..10.min(len as usize) {
                slot.prefix[j] = self.strings.units.get(key as usize + j)?;
            }
            slot.value = Some(atom([p[2], p[3], p[4]])?);
        }
        Ok(count)
    }
}
macro_rules! read_impl {($ty:ty)=>{
impl CapturedRead for $ty {
    type Value=Atom;
    fn root(&self)->Atom { // Root can only be written from an Atom.
        match self.data.root[0] {0=>Atom::Undefined,1=>Atom::Null,2=>Atom::Bool(false),3=>Atom::Bool(true),4=>Atom::Number(f64::from_bits(self.data.root[1] as u64|((self.data.root[2] as u64)<<32))),5=>Atom::String(self.data.root[1]),6=>Atom::Ref(self.data.root[1]),7=>Atom::BuiltinFunction(self.data.root[1]),_=>Atom::HostReference(self.data.root[1])}
    }
    fn header(&mut self,id:u32)->Result<Option<NodeHeader>,PeleError>{self.data.read_header(id)}
    fn own_fields(&mut self,id:u32,names:&[&str],out:&mut[Option<Atom>])->Result<(),PeleError>{self.data.own_fields(id,names,out)}
    fn property_prefixes(&mut self,id:u32,start:u32,out:&mut[PropertyPrefix<Atom>])->Result<usize,PeleError>{self.data.prefixes(id,start,out)}
    fn string_length(&mut self,id:u32)->Result<u32,PeleError>{Ok(self.data.strings.descriptors.get(id as usize)?[1])}
    fn with_string_chunks(&mut self,id:u32,visit:impl FnMut(&[u16])->Result<(),PeleError>)->Result<(),PeleError>{let [start,len]=self.data.strings.descriptors.get(id as usize)?;self.data.strings.units.chunks(start as usize,len as usize,visit)}
}
impl AdmissionRead for $ty {
    fn extent(&mut self)->Result<Extent,PeleError>{Ok(Extent{version:self.data.version,nodes:self.data.headers.len(),strings:self.data.strings.len()})}
    fn with_root<R>(&mut self,visit:impl FnOnce(&Atom)->Result<R,PeleError>)->Result<R,PeleError>{visit(&atom(self.data.root)?)}
    fn visit_string_lengths(&mut self,mut visit:impl FnMut(usize)->Result<(),PeleError>)->Result<(),PeleError>{for i in 0..self.data.strings.len(){visit(self.data.strings.descriptors.get(i)?[1] as usize)?;}Ok(())}
    fn header(&mut self,node:usize)->Result<Header,PeleError>{self.data.admission_header(node)}
    // Compatibility callbacks are bounded stack scratch, never used by selected validators.
    fn with_properties<R>(&mut self,node:usize,start:usize,visit:impl FnOnce(&[Property])->Result<R,PeleError>)->Result<R,PeleError>{
        let (start,n)=self.data.property_range(node,start)?;let count=n.min(PROPERTY_CHUNK);
        let mut scratch:[Property;PROPERTY_CHUNK]=std::array::from_fn(|_|Property{key:0,flags:0,value:Atom::Undefined});
        for (i,p) in scratch.iter_mut().take(count).enumerate(){*p=property(self.data.properties.get(start+i)?)?;}visit(&scratch[..count])
    }
    fn with_map_entries<R>(&mut self,node:usize,start:usize,visit:impl FnOnce(&[(Atom,Atom)])->Result<R,PeleError>)->Result<R,PeleError>{
        let (start,n)=self.data.items_range(node,start,3)?;let count=n.min(PROPERTY_CHUNK);
        let mut scratch:[(Atom,Atom);PROPERTY_CHUNK]=std::array::from_fn(|_|(Atom::Undefined,Atom::Undefined));
        for (i,(k,v)) in scratch.iter_mut().take(count).enumerate(){*k=atom(self.data.items.get(start+2*i)?)?;*v=atom(self.data.items.get(start+2*i+1)?)?;}visit(&scratch[..count])
    }
    fn with_set_values<R>(&mut self,node:usize,start:usize,visit:impl FnOnce(&[Atom])->Result<R,PeleError>)->Result<R,PeleError>{
        let (start,n)=self.data.items_range(node,start,4)?;let count=n.min(PROPERTY_CHUNK);let mut scratch:[Atom;PROPERTY_CHUNK]=std::array::from_fn(|_|Atom::Undefined);
        for (i,v) in scratch.iter_mut().take(count).enumerate(){*v=atom(self.data.items.get(start+i)?)?;}visit(&scratch[..count])
    }
    fn visit_properties(&mut self,node:usize,start:usize,remaining:usize,mut visit:impl FnMut(ScalarPropertyRef<'_>)->Result<(),PeleError>)->Result<usize,PeleError>{
        let (start,n)=self.data.property_range(node,start)?;let count=n.min(pages::PAGE);if count==0||count>remaining{return Err(fault())}
        self.data.properties.chunks(start,count,|values|{for p in values {visit(ScalarPropertyRef{key:p[0],flags:u8::try_from(p[1]).map_err(|_|fault())?,value:ScalarAtomRef::owned(atom([p[2],p[3],p[4]])?)})?;}Ok(())})?;Ok(count)
    }
    fn visit_map_entries(&mut self,node:usize,start:usize,remaining:usize,mut visit:impl FnMut(ScalarAtomRef<'_>,ScalarAtomRef<'_>)->Result<(),PeleError>)->Result<usize,PeleError>{
        let (start,n)=self.data.items_range(node,start,3)?;let count=n.min(pages::PAGE/2);if count==0||count>remaining{return Err(fault())}
        for i in 0..count {visit(ScalarAtomRef::owned(atom(self.data.items.get(start+2*i)?)?),ScalarAtomRef::owned(atom(self.data.items.get(start+2*i+1)?)?))?;}Ok(count)
    }
    fn visit_set_values(&mut self,node:usize,start:usize,remaining:usize,mut visit:impl FnMut(ScalarAtomRef<'_>)->Result<(),PeleError>)->Result<usize,PeleError>{
        let (start,n)=self.data.items_range(node,start,4)?;let count=n.min(pages::PAGE);if count==0||count>remaining{return Err(fault())}
        self.data.items.chunks(start,count,|values|{for v in values {visit(ScalarAtomRef::owned(atom(*v)?))?;}Ok(())})?;Ok(count)
    }
}
};}
read_impl!(FlatEpoch);
read_impl!(FlatWorking);
